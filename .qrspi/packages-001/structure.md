# packages-001 Structure

## Phase 1: Owner package read contract and schema support
- [x] Add additive schema for package list reads: package downloads, package repository links, package permissions, and package type coverage for Container, npm, RubyGems, Maven, NuGet, and Generic.
- [x] Add Rust/Postgres owner package list domain logic for users and organizations.
- [x] Enforce viewer visibility: anonymous public packages only; signed-in users can see self/direct package permission/repository permission packages; organization members can see internal/private org packages.
- [x] Support URL-backed filters for search, package type, visibility, linked-artifact tab, pagination, and most/least downloads sort.
- [x] Cover user/org package visibility and filters with focused API contract tests.

## Phase 2: Editorial owner profile packages UI
- [x] Wire `/{owner}?tab=packages` into the existing user profile shell with Packages tab selected.
- [x] Wire `/orgs/{org}/packages` and `/orgs/{org}?tab=packages` into the organization profile shell with Packages tab selected.
- [x] Render GitHub Packages and Linked artifacts tabs, type/visibility/sort dropdowns, search input, Clear filters link, package count heading, package rows, and filtered empty state.
- [x] Use Editorial primitives/tokens from `og.css` and avoid GitHub Primer palette/chrome.
- [x] Ensure all visible controls either work through concrete URLs/forms or explain placeholder behavior.

## Phase 3: Frontend API types, navigation, and focused component tests
- [x] Add typed package list DTOs and cookie-backed API clients.
- [x] Add navigation helper coverage for user and organization package filter URLs.
- [x] Add focused Vitest coverage for tabs, controls, rows, empty states, placeholder tab, links, and URL serialization.

## Phase 4: Verification and QA handoff
- [x] Run focused backend package contract tests.
- [x] Run focused frontend package component/navigation tests.
- [x] Run web typecheck.
- [x] Run mandatory Editorial banned-value scan on touched UI files.
- [x] Run `make check`.
- [x] Run `CARGO_INCREMENTAL=0 make test`.
- [x] Update `prd.json`, `build-progress.txt`, and `qa-hints.json` with packages-001 evidence.
