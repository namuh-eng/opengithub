# Structure Outline: discussions-004 Discussion Category Administration

**Ticket**: `discussions-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, existing `discussions-001` list/category reads, `discussions-002` creation/template parsing, `discussions-003` detail moderation metadata, repository settings patterns, repository file editor/commit patterns, and Editorial settings primitives.
**Date**: 2026-05-05

## Existing Baseline

`discussions-001` through `discussions-003` already make categories visible and useful to readers: list/category routes, creation metadata, YAML form parsing, polls, discussion detail, answer state, and category reassignment on individual discussions. `discussions-004` should add maintainer administration only: repository settings for sections/categories, category create/edit/delete with move destination, and `.github/DISCUSSION_TEMPLATE/*.yml` editing through repository file commits. It must preserve the Editorial design system and should not expand into organization-wide discussion policy, poll voting, general repository file browsing, or later discussion search indexing.

## Phase 1: Category Settings Contract - admin-ready reads and basic category writes

**Done**: [x]

**Scope**: Add the authenticated settings API and persistence needed to render/manage discussion categories and sections without shipping the full UI yet. Maintainers should be able to read sections/categories, create one category, edit metadata/format/section, and receive stable validation errors for limits and uniqueness.

**Key changes**:
- `crates/api/migrations/*_repository_discussion_category_admin.*.sql`: add `discussion_category_sections`, section/category `position`, `format`, `is_default`, `template_path`, and audit-friendly timestamps if absent; preserve existing category ids used by discussions.
- `crates/api/src/domain/discussions.rs`: add DTOs such as `DiscussionCategorySettingsView`, `DiscussionCategoryAdminItem`, `DiscussionCategorySectionItem`, `DiscussionCategoryFormat`, `CreateDiscussionCategoryRequest`, `UpdateDiscussionCategoryRequest`, and `DiscussionCategoryAdminViewer`.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/settings/discussions/categories`, `POST /categories`, and `PATCH /categories/{category_id}`; enforce admin/write permission, private repository privacy, archived/disabled guardrails, 25-category limit, unique normalized slug/name, emoji/name pair validation, format compatibility, and no-secret error envelopes.
- Side effects: write `repository_activity_events` and `audit_events` for create/update operations and keep `discussions-001/002/003` reader DTOs compatible with the new section/format fields.
- `web/src/lib/api.ts`: add typed category-settings DTOs and signed-cookie helpers without JS-side auth.
- `crates/api/tests/repository_discussions_contract.rs`: cover admin read, reader denial, private repository privacy, category create/edit, uniqueness, format validation, category limit, archived/disabled denial, audit/activity rows, and no session/OAuth/env leakage.

**Verification**: focused Rust category-admin contract tests against `TEST_DATABASE_URL`, `cd web && npx tsc --noEmit --pretty false`, focused Biome for touched web files, then full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Category Settings Page - rows, sections, and edit dialogs

**Done**: [ ]

**Scope**: Implement `/{owner}/{repo}/discussions/categories/edit` as an Editorial maintainer settings page backed by Phase 1. Admins should see section headers, category rows, New category/New section affordances, and working create/edit dialogs for category metadata and format.

**Key changes**:
- `web/src/app/[owner]/[repo]/discussions/categories/edit/page.tsx`: server-fetch repository metadata plus category settings data, render repository-safe unavailable/private states, and keep the Discussions tab/settings context active.
- `web/src/components/RepositoryDiscussionCategorySettingsPage.tsx`: render category/section list rows, emoji/name/description/format chips, counts, default markers, admin-only callouts, and no inert buttons using `.card`, `.btn`, `.chip`, `.input`, and type-ramp classes.
- `RepositoryDiscussionCategoryDialog`: implement create/edit form with emoji input, name input, description textarea, format selector for Announcement/Open-ended/Poll/Question and Answer, optional section assignment, validation summary, pending/error/success feedback, and concrete submit handlers through a same-origin proxy.
- `web/src/app/api/repos/[owner]/[repo]/settings/discussions/categories/...`: add cookie-forwarding route handlers for create/update where client interactivity needs same-origin mutations.
- Tests: cover settings page rendering, create/edit payloads, format labels, permission callouts, no `href="#"`, no inert click handlers, long text wrapping, mobile no-overflow, and mandatory Editorial banned-value guardrails.
- `web/tests/e2e/repository-discussions.spec.ts`: add focused browser smoke for opening dialogs, creating/editing a category, and screenshot `ralph/screenshots/build/discussions-004-phase2-category-settings.jpg` when a usable database URL is available.

**Verification**: focused Vitest, focused Playwright settings smoke when local DB credentials allow seeding, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Sections, Ordering, and Delete-With-Move - safe category restructuring

**Done**: [ ]

**Scope**: Complete structural administration: create/edit/delete sections, assign categories to sections, reorder section/category rows, delete a category only after choosing a destination category for moved discussions, and ensure format changes do not corrupt existing discussions.

**Key changes**:
- API routes: add `POST/PATCH/DELETE /settings/discussions/sections`, `PUT /settings/discussions/categories/order`, `PUT /settings/discussions/sections/order`, and `DELETE /settings/discussions/categories/{category_id}` with `move_to_category_id`.
- Domain validation: enforce one-section nesting only, stable positions, non-empty destination category on delete when discussions exist, prevent deleting the last category, preserve poll/form compatibility warnings, and record migration/audit events.
- `RepositoryDiscussionCategorySettingsPage`: add New section dialog, section rename/delete controls, move-to-section selector, reorder controls, delete confirmation dialog with destination category chooser, and clear warnings for category format changes.
- Side effects: update affected discussion category ids atomically on delete, preserve discussion numbers/timelines/forms/polls, refresh list/create/detail DTOs, and log `repository.discussion_category.*` activity/audit events.
- Tests: cover section CRUD, category section assignment, ordering persistence, delete-with-move, last-category denial, destination validation, discussion migration, format-change warning behavior, permission denial, no secret leakage, and browser no-dead-controls checks.
- Save screenshot `ralph/screenshots/build/discussions-004-phase3-section-delete.jpg` when seeded browser smoke is available.

**Verification**: focused Rust restructuring contracts, focused Vitest for section/delete dialogs, focused Playwright restructure smoke when DB credentials allow, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` if the wrapper is stable.

---

## Phase 4: Discussion Template File Editor - YAML commit and preview loop

**Done**: [ ]

**Scope**: Let maintainers edit category form templates through the repository file editor path for `.github/DISCUSSION_TEMPLATE/*.yml`. The editor should validate YAML, preview the parsed form schema, and commit/propose changes through existing Git object/commit infrastructure.

**Key changes**:
- API routes: add category-scoped template helpers such as `GET /settings/discussions/categories/{category_id}/template`, `PUT /template`, and `POST /template/preview`, or reuse existing file-content commit endpoints with a category-specific wrapper when already present.
- Domain/template logic: reuse `discussions-002` parser for preview, reject poll categories with templates, bound YAML size/fields/options, keep malformed schemas maintainer-visible, and ensure reader creation flows fall back safely.
- `web/src/app/[owner]/[repo]/discussions/categories/[categoryId]/template/page.tsx`: render syntax-highlighted YAML textarea/editor, parsed form preview, validation errors, branch/commit message fields, commit/propose-change controls, and links back to category settings using Editorial layout.
- Commit side effects: write/update `.github/DISCUSSION_TEMPLATE/{slug}.yml` on the default branch or create a proposed branch/commit where repository rules require it; update category `template_path` and cached form metadata after success.
- Tests: cover valid template preview, invalid YAML errors, poll-category rejection, commit payload/path normalization, proposed-change mode, cache refresh, create-flow fallback compatibility, no raw HTML reflection, no dead editor controls, and no secret leakage.
- Save browser screenshot `ralph/screenshots/build/discussions-004-phase4-template-editor.jpg`.

**Verification**: focused Rust template/commit contracts, focused Vitest for editor preview/submit behavior, focused Playwright editor smoke when seeded DB works, mandatory Editorial banned-value scan, then `make check && make test`; run direct Playwright equivalent if `make test-e2e` stalls.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `discussions-004` only after category/section management, delete migration, YAML template preview/commit, docs, screenshots, QA handoff, and PRD bookkeeping are complete. Do not implement organization-level discussion policy, poll voting, or unrelated repository settings here.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document category settings reads, category create/edit/delete/order, section create/edit/delete/order, template read/preview/commit, auth/privacy gates, validation envelopes, side effects, format compatibility, and no-secret guarantees.
- Final Rust tests: cover private repository privacy, archived/disabled repositories, malformed ids/slugs, category and section limits, unique names/slugs, delete migration, format/template compatibility, YAML parser bounds, audit/activity rows, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard traversal through settings rows, dialogs, confirmation flows, editor preview, commit/propose controls, server error display, long content wrapping, mobile no-overflow, no unsafe HTML, no `href="#"`, no inert buttons, and Editorial token compliance.
- `web/tests/e2e/repository-discussions.spec.ts`: full signed-session browser sweep for admin category settings, create/edit section/category, reorder, delete with move destination, YAML template preview/commit, reader denial, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/discussions-004/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `discussions-004.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/discussions-004-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
