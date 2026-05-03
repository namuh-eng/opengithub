# Structure Outline: releases-001 Repository Releases and Tags

**Ticket**: `releases-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, current placeholders in `web/src/app/[owner]/[repo]/releases/page.tsx` and `web/src/app/[owner]/[repo]/tags/page.tsx`, existing repository shell/navigation and compare helpers, Git ref contracts in `crates/api/src/domain/repositories.rs`, PR compare contracts in `crates/api/src/domain/pulls.rs`, and release references in `target-docs/content/repositories/releasing-projects-on-github/` plus `target-docs/content/rest/releases/`.
**Date**: 2026-05-03

## Phase 1: Releases Read Contract - list, detail, tags, assets, and permissions

**Done**: [x]

**Scope**: Add the Rust/Postgres API contract that lets viewers read public releases, latest release, release detail, repository tags, assets metadata, source archive links, and permission-sensitive private repository responses.

**Key changes**:
- `crates/api/migrations/`: add additive release tables for releases, release assets, release reactions, release contributors, release-download counters, and release audit metadata. Include repository id, tag name, target commit oid, title, markdown notes, rendered notes excerpt/cache metadata, draft/prerelease/latest flags, verified/tag-signature state, immutable/deleted timestamps, asset storage keys/checksums/content types, download counts, and unique active tag constraints.
- `crates/api/src/domain/releases.rs`: add DTOs for `RepositoryReleaseList`, `RepositoryReleaseSummary`, `RepositoryReleaseDetail`, `ReleaseAsset`, `ReleaseTagSummary`, `ReleaseReactionSummary`, `ReleaseContributorSummary`, and standard pagination.
- Routes under `/api/repos/{owner}/{repo}/releases`: list releases newest first, fetch `latest` as newest published non-prerelease, fetch by release id or tag, list tags, and expose source archive download metadata for zip/tar refs without leaking private repository refs to unauthorized viewers.
- Enforce visibility: anonymous users can read public releases; private repositories require read permission; drafts require write/admin permission; missing/private resources use non-leaky 404/403 behavior consistent with existing repository routes.
- Compute "Latest", "Pre-release", "Verified", short SHA, author, contributor avatars from existing users/refs/commits where possible. Keep Markdown rendering bounded and sanitized.
- Add `crates/api/tests/repository_releases_contract.rs` covering list ordering, latest resolution, prerelease/draft filtering, tag list, detail by tag/id, asset metadata, archive link shape, permission gates, private repository redaction, pagination, markdown sanitization, and standard error envelopes.

**Verification**: focused `repository_releases_contract` against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && DB_SSL=false make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Releases and Tags UI - list, detail, empty states, and responsive layout

**Done**: [ ]

**Scope**: Replace the Releases and Tags placeholders with real server-fetched Editorial pages. The release list/detail should match GitHub's information architecture while using the locked Editorial tokens and primitives.

**Key changes**:
- `web/src/lib/api.ts` and server-session helpers: add typed release/tag DTOs and cookie-backed fetch helpers that preserve forbidden/unavailable result states.
- `web/src/app/[owner]/[repo]/releases/page.tsx`, `web/src/app/[owner]/[repo]/releases/[tag]/page.tsx`, and `web/src/app/[owner]/[repo]/tags/page.tsx`: fetch API data server-side inside `AppShell` and existing repository navigation.
- Add `web/src/components/RepositoryReleasesPage.tsx`, `RepositoryReleaseDetailPage.tsx`, and `RepositoryTagsPage.tsx` with Releases/Tags tabs, release cards, tag/title, publish date, author avatar/name, short SHA, Verified chip, Latest/Pre-release chips, rendered Markdown notes, contributor avatar row, assets disclosure shell, source archive links, reaction bar shell, pagination, filtered empty states, and unavailable/forbidden states.
- Tags page should render chronological tag rows with commit links, verification status, release link when a tag has a release, and compare links using existing compare URL helpers.
- Use `.card`, `.btn`, `.chip`, `.tabs`, `.list-row`, `.av`, `.t-*`, `var(--ink-*)`, `var(--line)`, `var(--accent)`, and semantic chips only. No GitHub palette values, Primer imports, Octicons, nested cards, or inert `href="#"` links.
- Add focused Vitest coverage for list/detail/tags states, latest/prerelease chips, sanitized Markdown, asset disclosure initial state, empty/forbidden/unavailable states, concrete links, no dead visible controls, responsive content, and Editorial token/primitives guardrails.

**Verification**: focused release UI Vitest, web typecheck, mandatory Editorial banned-value scan, then `make check && make test`. Add a focused Playwright smoke if seeded release data is available.

---

## Phase 3: Assets, Reactions, Latest Redirect, and Compare Selector - all visible interactions work

**Done**: [ ]

**Scope**: Wire the visible interactive controls to real server-confirmed behavior: asset/source downloads, reaction toggles, latest navigation, assets disclosure, and compare tag selection.

**Key changes**:
- Add backend endpoints for asset download authorization/counting, source archive generation/download redirect, reaction toggle/delete for authenticated viewers, and compare-target metadata for releases/tags.
- Integrate with existing archive/job/storage patterns where available. Asset downloads should authorize before returning a signed/local URL, increment download counters transactionally, and never expose S3 storage keys directly.
- Add same-origin Next.js route handlers or server actions for release reactions and asset/download redirects without JS-side auth.
- Convert assets and reactions to client-backed components where needed: disclosure opens without layout shift, reaction buttons submit and refresh from server state, unauthenticated viewers see sign-in/disabled state, and failed mutations roll back without local-only success.
- Compare control opens a searchable tag selector and navigates to `/{owner}/{repo}/compare/{base}...{head}` using existing compare helpers; no menu item should be inert.
- Add Rust tests for asset auth/counting, archive link generation, reaction toggle idempotency, unauthenticated reaction rejection, private asset redaction, and compare target validation.
- Add Vitest and Playwright coverage for disclosure, asset links, reaction success/error/anonymous states, latest redirect/detail page, compare selector navigation, mobile no-overflow, and no dead controls. Save `ralph/screenshots/build/releases-001-phase3-*.jpg`.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` when the local database/dev servers are stable.

---

## Phase 4: Release Management Mutations - create, edit, publish, delete, assets, and audit

**Done**: [ ]

**Scope**: Complete the CRUD surface expected of repository releases. Collaborators with write permission can create draft/published releases, edit release metadata, upload/remove assets, publish drafts, delete releases, and keep tags/source files consistent with repository policy.

**Key changes**:
- Extend Rust release domain with create/update/delete/publish endpoints, release asset upload/delete metadata endpoints, and audit rows. Keep upload storage local/S3-pluggable using the same storage contract shape as Pages/packages artifacts.
- Validate tag names against existing Git refs or allow creation from a selected target commit/ref when authorized; reject duplicate active releases for a tag; enforce archived repository and immutable release guardrails.
- Add UI affordances only when viewer permissions allow them: New release, Edit, Delete, Publish draft, Upload asset, Remove asset, and validation/success/error feedback. Read-only viewers should not see live mutation controls.
- Add release form components with tag picker, target selector, title, notes Markdown preview, prerelease/draft toggles, asset upload list, and destructive confirmations. Every form submits to the Rust API and refreshes from returned server state.
- Add tests for write/admin gates, draft visibility, duplicate tag validation, publish semantics, asset upload/delete metadata, audit redaction, immutable release restrictions, archived repository blocking, and no plaintext/storage-key leakage.
- Add UI and E2E tests for create/edit/delete/publish/asset flows, rejected mutation rollback, permission-gated controls, keyboard-accessible confirmations, and mobile layout.

**Verification**: focused Rust mutation contract, focused Vitest, focused Playwright management smoke, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` if seeded auth state can exercise write permissions.

---

## Phase 5: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `releases-001` only after read UI, interactions, management mutations, docs, screenshots, and QA handoff are verified. This phase should not expand into packages, marketplace publishing, release discussions, security advisories, or organization-level release policies.

**Key changes**:
- Add `/docs/api` coverage for release list/detail/latest/tags, create/update/delete/publish, assets upload/download/delete, source archive downloads, reaction toggle/delete, permissions, validation errors, pagination, private repository redaction, and local/S3 storage notes.
- Extend `qa-hints.json` with deeper QA targets: real S3 asset storage and signed URLs, large asset limits, archive generation correctness, Markdown sanitization/XSS, concurrent release edits, immutable release/tag policy, private repository leakage, asset download counters, reaction race conditions, and accessibility traversal.
- Ensure every visible button/link/form has concrete behavior or an accessible disabled state; verify keyboard navigation through Releases/Tags tabs, release cards, assets disclosure, reaction controls, compare selector, pagination, release forms, asset upload controls, and destructive confirmations.
- Save final screenshots under `ralph/screenshots/build/releases-001-final-*.jpg` for list, detail/latest, tags, empty, mutation, anonymous/disabled reaction, mobile, and forbidden states.
- Run the mandatory Editorial banned-value scan before commit and fix any touched-file regressions.
- Update `build-progress.txt`, `.qrspi/releases-001/structure.md`, and `prd.json`; set `releases-001.build_pass=true` only after all implementation phases are complete and verified; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when available, browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
