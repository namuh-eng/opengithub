# Structure Outline: discussions-002 Discussion Creation

**Ticket**: `discussions-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, existing `discussions-001` list/category/vote contracts, repository shell/navigation patterns in `web/src/lib/navigation.ts`, Markdown sanitization and attachment patterns from issues/PR comments, notification fanout from `notifications-001`, and community links/category rail behavior from `discussions-001`.
**Date**: 2026-05-05

## Existing Baseline

`discussions-001` already provides repository discussion categories, list/category routes, filtering, upvote writes, active repository navigation, helpful resources/community links, and authenticated API permission checks. `discussions-002` should add the creation flow only: category chooser, category-aware new form, Markdown preview, similar-search acknowledgement, optional YAML discussion form answers, optional polls, attachments, subscriptions, notifications, and redirect to the new detail URL. It must keep the Editorial design system and should not expand into discussion detail timelines, comments, answer marking, category administration, moderation, or poll voting.

## Phase 1: Creation Contract and Category Form Data - chooser-ready API

**Done**: [x]

**Scope**: Add the persistence and authenticated API contracts needed for the new-discussion flow without rendering the UI yet. The API should expose creation metadata for category cards and selected-category forms, parse discussion template YAML from repository Git content when present, provide safe fallback fields for invalid templates, and create normal non-poll discussions with required similar-search acknowledgement.

**Key changes**:
- `crates/api/migrations/*_repository_discussion_creation.*.sql`: add `discussion_category_forms`, `discussion_form_answers`, `discussion_attachments` metadata if not already present, plus indexes by repository/category/discussion and bounded attachment ownership fields. Reuse existing `discussion_categories`, `discussions`, `discussion_comments`, subscriptions, notifications, and activity tables.
- `crates/api/src/domain/discussions.rs`: add DTOs such as `DiscussionCreationView`, `DiscussionCategoryChoice`, `DiscussionFormDefinition`, `DiscussionFormField`, `DiscussionSimilarSearch`, `CreateDiscussionRequest`, `CreateDiscussionResponse`, and `DiscussionAttachmentDraft`.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/discussions/new`, `GET /api/repos/{owner}/{repo}/discussions/new/categories/{slug}`, and `POST /api/repos/{owner}/{repo}/discussions`; enforce authenticated repository read/write permission, disabled/archived guardrails, private repository privacy, required title/body/form fields, similar-search acknowledgement, and no-secret error envelopes.
- Git/template parsing: read `.github/DISCUSSION_TEMPLATE/*.yml` from the repository default branch when available; validate supported field types, strip unsafe Markdown/HTML from descriptions, bound field counts/options, and fall back to generic title/body composer on invalid YAML.
- Creation side effects: insert discussion row, initial body comment, form answers, viewer subscription, repository activity event, and notification rows for subscribed maintainers/watchers with bounded metadata only.
- `crates/api/tests/repository_discussion_creation_contract.rs`: seed categories, repository permissions, valid/invalid YAML templates, private repositories, disabled policy, archived repositories, and similar-search state; verify metadata DTOs, validation errors, successful normal discussion creation, answer persistence, initial comment, subscriptions, notifications, activity events, redirect href, and no session/OAuth/env/template secret leakage.
- `web/src/lib/api.ts`: add typed creation DTOs and cookie-backed fetch/create helpers without adding client-side auth.

**Verification**: focused Rust contract tests against `TEST_DATABASE_URL`, `cd web && npx tsc --noEmit --pretty false`, focused Biome for touched web files, then full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Category Chooser - selectable cards and preselected routing

**Done**: [ ]

**Scope**: Implement `/{owner}/{repo}/discussions/new/choose` and the preselected-category entry behavior. Users should see category cards with emoji, name, description, answer-enabled badges, counts, and working Get started links. Direct `/{owner}/{repo}/discussions/new?category={slug}` should skip the chooser and load the selected form route data.

**Key changes**:
- `web/src/app/[owner]/[repo]/discussions/new/choose/page.tsx`: server-fetch creation metadata, render repository-safe unavailable/disabled/private states, and keep the Discussions tab active.
- `web/src/components/RepositoryDiscussionCategoryChooser.tsx`: render Editorial category cards, answer/poll/generic badges, recent activity/count metadata, helpful resources sidebar, first-time community reminder, and no inert controls.
- `web/src/lib/navigation.ts`: add `repositoryDiscussionChooseCategoryHref` and extend `repositoryNewDiscussionHref` query handling for selected category, similar-search query, and return-safe redirects.
- Category permissions: hide draft/private template details from unauthorized callers, show disabled organization policy callout when Discussions cannot be created, and keep links concrete rather than `href="#"`.
- `web/tests/repository-discussion-create-page.test.tsx`: cover chooser cards, answer-enabled badge, poll badge, disabled/empty states, Get started links, preselected-category hrefs, active nav, no unsafe HTML, no dead links, long category wrapping, mobile no-overflow, and Editorial banned-value guardrails.
- `web/tests/e2e/repository-discussion-create.spec.ts`: focused browser smoke for chooser navigation to selected category and screenshot `ralph/screenshots/build/discussions-002-phase2-chooser.jpg` when a usable database URL is available.

**Verification**: focused Vitest, focused Playwright chooser smoke when local DB credentials allow seeding, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Markdown Composer and Similar-Search Acknowledgement - generic discussion creation

**Done**: [ ]

**Scope**: Implement the normal new-discussion form for categories that use the generic title/body composer or invalid YAML fallback. Markdown preview must render without creating a discussion, similar-search must derive from the typed title, acknowledgement is required, attachments use real upload metadata, and submit redirects to the new discussion detail URL.

**Key changes**:
- `web/src/app/[owner]/[repo]/discussions/new/page.tsx`: server-fetch selected category form data, redirect to chooser when category is absent, and render the unavailable fallback without leaking private repo data.
- `web/src/components/RepositoryDiscussionCreatePage.tsx`: render selected category summary, Choose a different category link, title input, Markdown write/preview tabs, composer toolbar, attachment dropzone/file input, similar-search link and required acknowledgement checkbox, Start discussion button, success/error feedback, and helpful resources sidebar using Editorial primitives/tokens only.
- `web/src/app/api/repos/[owner]/[repo]/discussions/route.ts` or existing proxy pattern: forward create requests with signed cookies and standardized error envelopes; include attachment draft IDs when present.
- Markdown preview: reuse the existing sanitized Markdown renderer/server endpoint pattern from issue/PR/comment surfaces; ensure preview does not persist discussion/comment rows.
- Attachment integration: create bounded attachment metadata and S3/object references through existing upload helpers where available; reject oversized/unsupported files with stable validation envelopes.
- Tests: cover required title/body/acknowledgement validation, Markdown preview sanitization, attachment validation, successful generic create, redirect href, duplicate submit prevention, no dead toolbar/dropzone controls, keyboard labels, and no raw HTML reflection.
- Save browser screenshot `ralph/screenshots/build/discussions-002-phase3-generic-create.jpg`.

**Verification**: focused Rust create/preview/attachment contracts, focused Vitest, focused Playwright generic-create smoke, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 4: YAML Category Forms and Poll Creation - form-specific creation paths

**Done**: [ ]

**Scope**: Add category-specific YAML form field rendering and poll-category creation. Required YAML fields should validate before submit and persist `discussion_form_answers`; poll categories should render question/options controls instead of YAML fields and persist poll/options with validation.

**Key changes**:
- `crates/api/migrations/*_repository_discussion_polls.*.sql`: add `discussion_polls` and `discussion_poll_options` if absent, indexed by discussion and option position; keep poll voting out of scope.
- API validation: extend `POST /api/repos/{owner}/{repo}/discussions` to accept `formAnswers` or `poll`, reject polls mixed with category forms, validate required YAML fields, option counts, duplicate/blank options, and poll question length.
- `RepositoryDiscussionCreatePage`: render supported YAML field types from `DiscussionFormDefinition` with labels, descriptions, required markers, select/dropdown options, checkboxes, textareas, validation summaries, and answered-category helper text.
- `RepositoryDiscussionPollForm`: render poll question/options controls, add/remove option buttons, minimum/maximum option rules, and disabled submit state until valid.
- Persistence and side effects: store form answers and poll rows in the same create transaction as the discussion/comment/subscription/notification/activity rows; redirect to detail after success.
- Tests: cover valid YAML form creation, invalid YAML fallback remains generic, missing required form fields, unsupported YAML fields ignored safely, poll creation, poll validation, poll/form conflict rejection, sanitized descriptions, no plaintext attachment/template leakage, and mobile no-overflow.
- Save browser screenshot `ralph/screenshots/build/discussions-002-phase4-form-poll.jpg`.

**Verification**: focused Rust form/poll contracts, focused Vitest for form and poll variants, focused Playwright form/poll smoke, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` if the wrapper is stable.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `discussions-002` only after chooser, generic creation, YAML forms, poll creation, preview, attachments, side effects, docs, screenshots, QA handoff, and PRD bookkeeping are complete. Do not implement discussion detail timelines/comments, answer marking, poll voting, moderation, or category management here.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document discussion creation metadata, selected-category form metadata, create discussion, preview/attachment behavior if endpoint-backed, auth/privacy gates, validation envelopes, template fallback, poll restrictions, side effects, and no-secret guarantees.
- Final Rust tests: cover private repository privacy, disabled policy, archived repositories, malformed category slugs, missing similar-search acknowledgement, required YAML fields, invalid YAML fallback, sanitized Markdown preview/body, attachment bounds, poll validation, subscription/notification/activity writes, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard traversal through chooser/composer/toolbar/preview/attachments/forms/polls, successful submit redirects, server error display, no `href="#"`, no inert click handlers, long text wrapping, mobile no-overflow, and Editorial token compliance.
- `web/tests/e2e/repository-discussion-create.spec.ts`: full signed-session browser sweep for chooser, category-specific form, generic fallback, Markdown preview, similar-search acknowledgement, attachment validation, poll creation, successful submit redirect, disabled states, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/discussions-002/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `discussions-002.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/discussions-002-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
