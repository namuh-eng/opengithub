# Structure Outline: packages-002 Package Detail Pages

**Ticket**: `packages-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/wf-profiles.jsx`, existing owner package list implementation in `crates/api/src/domain/packages.rs`, `web/src/components/OwnerPackagesPage.tsx`, current package metadata routes in `crates/api/src/routes/packages.rs`, and package docs under `target-docs/content/packages/learn-github-packages/`.
**Date**: 2026-05-03

## Phase 1: Owner-Scoped Detail API Contract - readable package, versions, blobs, README/about, and permissions

**Done**: [x]

**Scope**: Add the Rust/Postgres read contract for `/{owner}/{package_type}/{package_name}` and `/orgs/{org}/packages/{package_type}/{package_name}` without depending on GitHub APIs. Public packages are anonymous-readable, private/internal packages require package or linked-repository read permission, and admin capability is returned only when the viewer can manage the package.

**Key changes**:
- `crates/api/migrations/`: add only the metadata needed by detail rendering if missing, such as package version digest/platform labels, package blob/layer records, README/about overrides, and immutable version lookup indexes. Preserve existing `packages`, `package_versions`, `package_downloads`, `package_repository_links`, and `package_permissions` contracts.
- `crates/api/src/domain/packages.rs`: add `PackageDetail`, `PackageDetailVersion`, `PackageBlobSummary`, `PackageInstallCommand`, `PackageAboutContent`, `PackageAdminState`, and `PackageDetailQuery` DTOs plus owner-scoped lookup helpers by owner kind, package type, package name, optional version/tag/digest, and actor.
- `crates/api/src/routes/users.rs` and `crates/api/src/routes/organizations.rs`: expose owner-scoped detail endpoints such as `GET /api/users/{username}/packages/{package_type}/{package_name}` and `GET /api/orgs/{org}/packages/{package_type}/{package_name}` with the standard JSON error envelope.
- `crates/api/src/routes/packages.rs`: keep existing repository package routes working; do not regress repo-scoped package create/version APIs.
- `crates/api/tests/package_detail_contract.rs`: cover public read, private/internal redaction, package permission grants, linked repository permission grants, latest version selection, digest/version selection, blob/platform metadata, README/about fallback, admin-state calculation, missing package/type errors, and no storage-key leakage.

**Verification**: focused `cargo test -p opengithub-api --test package_detail_contract`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Detail Pages - header, install card, recent versions, and README/about

**Done**: [x]

**Scope**: Render real server-fetched package detail pages for user- and organization-scoped canonical routes. The page should use the owner/profile shell when owner-scoped and repository context links when a source repository is linked, while staying fully inside the Editorial visual system.

**Key changes**:
- `web/src/lib/api.ts` and server-session helpers: add typed package detail DTOs and cookie-backed fetch helpers for user and organization package detail endpoints, preserving forbidden, not-found, and unavailable states.
- `web/src/app/[owner]/[package_type]/[package_name]/page.tsx`: render user-scoped package detail pages.
- `web/src/app/orgs/[org]/packages/[package_type]/[package_name]/page.tsx`: render organization-scoped package detail pages.
- `web/src/components/PackageDetailPage.tsx`: add the Editorial package header with type icon, package name, short version/digest metadata, visibility badge, Latest badge, linked repository/publisher links, admin Settings entry point when permitted, install command card, recent versions table, blob/platform summary, and README/about content or empty-about state.
- `web/src/components/OwnerPackagesPage.tsx`: keep existing list links aligned with the new detail routes and ensure rows remain concrete links.
- `web/tests/package-detail-page.test.tsx`: cover populated, latest, selected version, missing README/about, forbidden/unavailable, user/org canonical links, admin button visibility, no dead links/buttons, and Editorial token/primitives guardrails.
- `web/tests/e2e/package-detail.spec.ts`: seed a visible package, open from the owner Packages list, verify header/install/recent versions/about sections, check mobile no-overflow, and save `ralph/screenshots/build/packages-002-phase2-detail.jpg`.

**Verification**: focused Vitest, focused Playwright smoke when the local DB/dev server is stable, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 3: Version Selection, Copy Commands, Immutable Version Details, and Download Metadata

**Done**: [x]

**Scope**: Make every interactive detail-page control real. Selecting a version updates metadata and install commands, copy buttons write to the clipboard with temporary success feedback, version rows route to immutable detail URLs, and registry/download metadata routes record `package_downloads` only when an actual registry/source download endpoint is hit.

**Key changes**:
- `crates/api/src/domain/packages.rs`: add immutable version lookup by id/tag/digest, digest-pull metadata, archive/blob download authorization metadata, and a bounded package-download recording helper for real download endpoints only.
- `crates/api/src/routes/users.rs`, `crates/api/src/routes/organizations.rs`, and/or `crates/api/src/routes/packages.rs`: expose selected-version detail and download metadata routes while preserving permission checks and storage-key redaction.
- `web/src/components/PackageDetailInteractions.tsx`: add client-side version selector state from server-provided version DTOs, copy command buttons with success/error states, digest-pull command toggles, and keyboard-accessible disclosure for platform/blob variants.
- `web/src/app/...`: support query or path selection for immutable version detail links without local-only state.
- Extend `crates/api/tests/package_detail_contract.rs`: cover selected version resolution, digest-pull links, download-count writes only from explicit download endpoints, private package denial, and malformed digest/tag validation.
- Extend `web/tests/package-detail-page.test.tsx` and `web/tests/e2e/package-detail.spec.ts`: cover version switching, clipboard success fallback, immutable version link navigation, digest command rendering, no optimistic-only metadata, and browser screenshots.

**Verification**: focused Rust/Vitest/Playwright checks, mandatory Editorial banned-value scan, then `make check && make test`; run full `make test-e2e` when the shared migration state allows it.

---

## Phase 4: Admin Settings Entry Surface - access, visibility, repository link, and provenance overview

**Done**: [x]

**Scope**: Ensure the visible package Settings entry opens a real admin-only surface instead of a placeholder. This phase does not need full package registry mutation parity, but it must show current access/visibility/source-link state, concrete navigation, safe disabled explanations for not-yet-supported writes, and non-admin redaction.

**Key changes**:
- `crates/api/src/domain/packages.rs`: add package settings read DTOs for visibility, owner, linked repositories, explicit package permissions, inherited repository access summary, recent package activity, and future registry write capability flags.
- `crates/api/src/routes/users.rs` and `crates/api/src/routes/organizations.rs`: expose admin-gated settings read endpoints for package settings, returning 403 without private metadata for non-admin viewers.
- `web/src/app/[owner]/[package_type]/[package_name]/settings/page.tsx` and `web/src/app/orgs/[org]/packages/[package_type]/[package_name]/settings/page.tsx`: render the settings entry surface.
- `web/src/components/PackageSettingsPage.tsx`: use Editorial cards/list rows for access, visibility, linked repository, Actions workflow access, and danger-zone placeholders with accessible disabled buttons only when the write API belongs to a later feature.
- Extend tests for admin-only access, non-admin redaction, settings link visibility, concrete back/detail links, disabled-control reasons, and no package secret/storage leakage.

**Verification**: focused Rust/Vitest/Playwright settings smoke, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 5: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `packages-002` only after API contracts, UI pages, interactions, admin entry, docs, screenshots, and QA handoff are verified. Do not implement the full OCI push/pull registry protocol here; that belongs to `packages-003`.

**Key changes**:
- `web/src/lib/api-docs.ts`: document owner-scoped package detail, selected-version, immutable version, download metadata, and settings read endpoints, including auth/visibility, digest/platform fields, storage-key redaction, and download-count semantics.
- `qa-hints.json`: append package detail QA targets covering private/internal package leakage, actual registry download accounting, long names/digests, multi-platform install commands, missing README/about states, clipboard behavior, admin settings redaction, linked repository permission inheritance, and accessibility traversal.
- `web/tests/e2e/package-detail.spec.ts`: expand final smoke to cover user package detail, org package detail, selected version/digest, settings admin entry, forbidden state, empty README/about, desktop/mobile screenshots, and dead-control scanning.
- `ralph/screenshots/build/`: save final screenshots such as `packages-002-final-user-detail.jpg`, `packages-002-final-org-detail.jpg`, `packages-002-final-version-detail.jpg`, `packages-002-final-settings.jpg`, `packages-002-final-forbidden.jpg`, and `packages-002-final-mobile.jpg`.
- `build-progress.txt`, `.qrspi/packages-002/structure.md`, and `prd.json`: record verification evidence and set `packages-002.build_pass=true` only after all phases are complete and verified; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, mandatory Editorial banned-value scan, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, same-env `make test`, and `make test-e2e` when the local migration state can run; otherwise document the precise migration blocker and include a fresh-DB Playwright pass if needed.
