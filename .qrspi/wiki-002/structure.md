# Structure Outline: wiki-002 Wiki Editing and Publishing

**Ticket**: `wiki-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, existing `wiki-001` structure and implementation, `crates/api/src/domain/wiki.rs`, `crates/api/src/routes/repositories.rs`, `web/src/components/RepositoryWikiPage.tsx`, `web/src/components/WikiPageList.tsx`, and `web/src/lib/navigation.ts`.
**Date**: 2026-05-06

## Existing Baseline

`wiki-001` completed the read surface: repository wiki persistence, sanitized Markdown rendering, Home and slug reads, page list/TOC expansion, sidebar/footer rendering, clone URL metadata, docs, browser screenshots, and E2E seed coverage. `wiki-002` should add creation/editing/preview/save flows on top of that baseline while keeping Rust API/session authority, Editorial UI primitives, and local Git-backed publishing semantics. Do not introduce GitHub visual styling, Primer/Octicons, JS-side auth, external GitHub APIs, a browser Markdown renderer as source of truth, or a full remote Git smart-HTTP implementation beyond the local wiki publishing contract required here.

## Phase 1: Wiki Write Contract and Local Git Publisher - create/edit/save works through Rust

**Done**: [x]

**Scope**: Add the backend mutation contract for creating and editing wiki pages, including validation, permissions, revision rows, rendered cache invalidation, audit/activity events, and a local bare wiki repository commit path. This phase should be API-first and independently testable without replacing the reader UI.

**Key changes**:
- `crates/api/migrations/*_repository_wiki_write.*.sql`: add only additive columns/tables needed for write tracking, such as `wiki_git_commits`, optional `wiki_assets`, latest pushed commit metadata, and indexes for repository/page/revision lookup. Preserve existing `wiki_pages`, `wiki_page_revisions`, `wiki_repositories`, and `rendered_markdown_cache` contracts.
- `crates/api/src/domain/wiki.rs`: add request/response DTOs such as `WikiPageSaveRequest`, `WikiPagePreviewRequest`, `WikiPageMutationResult`, `WikiEditMode`, `SupportedMarkupFormat`, and `WikiGitCommitSummary`.
- `crates/api/src/domain/wiki_git.rs` or a scoped module: implement local bare repository initialization and commit publishing for `{repo}.wiki.git`, backed by `wiki_repositories.git_storage_path`/`default_branch`. The DB write and local commit metadata should stay consistent on success.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/wiki/_pages`, `GET /api/repos/{owner}/{repo}/wiki/{slug}/edit`, `POST /api/repos/{owner}/{repo}/wiki/pages`, `PATCH /api/repos/{owner}/{repo}/wiki/{slug}`, and `POST /api/repos/{owner}/{repo}/wiki/preview`.
- Validation behavior: require editor permission, enabled wiki, supported edit mode/extension, non-empty title, safe slug/path, non-empty body, non-empty commit message, stale `expectedRevisionId` conflict handling, and repository-safe 404s for unauthorized private reads.
- Publishing behavior: create/update `wiki_pages`, append `wiki_page_revisions`, insert `wiki_git_commits`, invalidate/update rendered Markdown/TOC caches, write repository activity and audit events, and redirect clients to the rendered page href.
- `crates/api/tests/repository_wiki_write_contract.rs`: cover create, edit, preview without persistence, invalid titles/extensions/messages, reader denial, disabled wiki denial, stale revision conflict, sidebar/footer edits, local Git commit metadata, cache refresh, activity/audit events, and no secret leakage.
- `web/src/lib/api.ts` and server helpers: add typed wiki list/edit/preview/save helpers with signed-cookie forwarding and no JS auth library.

**Verification**: focused Rust write contract tests against `TEST_DATABASE_URL` when available, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, `cd web && npx tsc --noEmit --pretty false`, focused Biome for touched web types, then full `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Pages Index and New Page Entry - editors can discover and start pages

**Done**: [x]

**Scope**: Build `/{owner}/{repo}/wiki/_pages` and route the existing New Page affordance to a real empty editor. The pages index should be backed by the Phase 1 list contract and expose concrete links for page detail/edit creation without dead controls.

**Key changes**:
- `web/src/app/[owner]/[repo]/wiki/_pages/page.tsx`: server-fetch repository metadata and wiki page list, preserve repository-safe unavailable states, and keep the Wiki tab active.
- `web/src/app/[owner]/[repo]/wiki/_new/page.tsx`: render the empty editor only when the viewer can edit; readers should see a permission-aware unavailable state or redirect-safe error.
- `web/src/components/RepositoryWikiPagesIndex.tsx`: render Editorial `Pages` heading, New Page button for editors, document-icon list rows sorted by title, page-title links, edit links for editors, and relative `Last updated` timestamps.
- `web/src/components/RepositoryWikiEditor.tsx`: introduce the shared editor shell with title input, edit mode dropdown, Markdown textarea/code editor, toolbar including image insert controls, Write/Preview tabs, edit message input, Save Page button, and validation flash area. In this phase, only new-page draft state and preview are wired; save can call the Phase 1 create endpoint if already available.
- `web/src/lib/navigation.ts`: add href helpers for pages index, new page, edit page, history-safe revision links, and rendered-page redirects.
- Tests: cover pages index sorting, permissioned New Page button, row links, empty list state, new-page editor fields, mode dropdown, toolbar image insertion into Markdown, preview request payload, no `href="#"`, no inert buttons, long titles wrapping, and Editorial guardrails.
- Browser smoke: load `/wiki/_pages`, open New Page, type title/body/message, insert an image Markdown reference, run Preview, verify preview feedback, check no dead controls, and save `ralph/screenshots/build/wiki-002-phase2-pages-new.jpg`.

**Verification**: focused Vitest and browser smoke, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Edit Existing Page and Preview - selected pages round-trip through the editor

**Done**: [x]

**Scope**: Wire selected page editing from the reader and pages index. Editors should open an existing page, preview unsaved Markdown, save with a commit message, and land back on the rendered page with refreshed content.

**Key changes**:
- `web/src/app/[owner]/[repo]/wiki/[...slug]/_edit` is not a valid App Router child for catch-all; use a route shape that Next supports, such as `web/src/app/[owner]/[repo]/wiki-edit/[...slug]/page.tsx`, or a route group that avoids catch-all child conflicts while preserving public hrefs from the API.
- `RepositoryWikiPage`: ensure Edit buttons point to the supported edit route, page list edit affordances are concrete, and missing-page New Page CTAs prefill the intended slug/title.
- `RepositoryWikiEditor`: hydrate existing title/body/mode/latest revision, submit Preview without persistence, submit Save with `expectedRevisionId`, display pending/error/success states, and navigate to the returned rendered page href.
- Same-origin Next proxy routes if needed: forward preview/create/update JSON requests to Rust with cookies, preserve structured errors and status codes, and avoid duplicating auth logic.
- Markdown preview: render only sanitized HTML returned by Rust; never trust client-side Markdown output as the saved preview.
- Tests: cover existing edit hydration, preview HTML rendering, save PATCH payload, stale conflict display, validation flash, reader-disabled edit state, missing page prefill, sidebar/footer edit links, button types, no placeholder hrefs, and Editorial guardrails.
- Browser smoke: load a seeded wiki page, open Edit, change body and commit message, Preview, Save, verify redirect to rendered page with changed content, check no `href="#"`, no horizontal overflow, and save `ralph/screenshots/build/wiki-002-phase3-edit-save.jpg`.

**Verification**: focused Rust update/preview contracts, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 4: Markup Modes, Image References, and Publishing Edge Cases - validation matches product expectations

**Done**: [x]

**Scope**: Complete editor capability details: supported markup modes/extensions, URL-based image insert references, sidebar/footer creation, local Git default-branch publishing boundaries, duplicate/renamed slugs, and live page visibility only after a successful wiki commit.

**Key changes**:
- `crates/api/src/domain/wiki.rs`: centralize title-to-slug/path normalization, supported markup lookup, image reference extraction, duplicate-title handling, rename semantics, and commit branch enforcement.
- `supported_markup_formats`: seed or expose Markdown plus any already-supported markup extensions; unsupported modes return structured validation errors rather than silently saving.
- `wiki_assets` references: record linked image URLs/alt text when the toolbar inserts or saved Markdown contains images; do not upload binary assets unless an existing storage pipeline already supports it.
- Local Git behavior: ensure only commits to the wiki default branch update `wiki_pages.latest_revision_id` and rendered cache; non-default branch pushes may be recorded as git commits but must not become live pages.
- Editor UI: make the mode dropdown authoritative, surface duplicate/invalid title errors, preserve draft input after failed save, and show clear success/error flashes.
- Tests: cover supported/unsupported modes, image reference extraction, duplicate title/slug conflict, rename updates href/path, sidebar/footer save behavior, non-default branch no-live-update guard, cache invalidation, audit/activity events, keyboard navigation through toolbar/tabs, and mobile no-overflow.
- Browser smoke: exercise image insert, invalid title error, unsupported mode error if exposed, then successful save and rendered image Markdown link visibility; save `ralph/screenshots/build/wiki-002-phase4-markup-images.jpg`.

**Verification**: focused contract/unit tests, focused browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 5: Docs, E2E, QA Handoff, and Build-Pass Bookkeeping

**Done**: [x]

**Scope**: Finish `wiki-002` only after pages index, create, edit, preview, save, local Git-backed publishing, markup validation, image reference tracking, docs, screenshots, and QA handoff are complete.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document wiki pages index, edit fetch, preview, create, update, validation errors, permissions, stale revision conflicts, supported markup formats, local Git publishing semantics, activity/audit side effects, and image reference tracking.
- Final Rust tests: cover end-to-end create/edit/save flows, permission boundaries, disabled/private repositories, stale revisions, local Git repo metadata, cache refresh, activity/audit rows, unsupported markup, unsafe Markdown, image links, and no credential/env leakage.
- Final frontend tests: cover pages index, empty editor, existing editor, Write/Preview tabs, toolbar image insert, save success/error states, redirect behavior, no placeholder hrefs, no inert click handlers, long content wrapping, keyboard traversal, and Editorial token compliance.
- `web/tests/e2e/repository-wiki-editing.spec.ts`: signed-session sweep for pages index, create page, preview, save, edit existing page, conflict/error state when practical, screenshot capture, and mobile no-overflow.
- `qa-hints.json`: append targets for real local Git repository contents, concurrent edit conflicts, title/slug normalization, sidebar/footer special pages, unsafe Markdown/images, permission leakage, very large pages, keyboard-only editing, and failed Git commit rollback.
- `build-progress.txt`, `.qrspi/wiki-002/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `wiki-002.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/wiki-002-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, JSON validation for `prd.json` and `qa-hints.json`, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan:

```bash
rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'
```
