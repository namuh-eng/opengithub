# Structure Outline: wiki-001 Repository Wiki Reader

**Ticket**: `wiki-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-shell.jsx`, existing repository shell/navigation in `web/src/components/RepositoryShell.tsx` and `web/src/lib/navigation.ts`, current placeholder route at `web/src/app/[owner]/[repo]/wiki/page.tsx`, repository settings wiki feature flag from `settings-001`, Markdown rendering/cache contracts in `crates/api/src/domain/markdown.rs`, and security policy Markdown reader patterns in `RepositorySecurityPolicyPage`.
**Date**: 2026-05-06

## Existing Baseline

Repository navigation already exposes a Wiki tab and repository General settings include `wiki_enabled`, but `/{owner}/{repo}/wiki` is only a placeholder. The implementation should turn Wiki into a real repository-owned read surface using the existing Rust API/session authority, SQLx migrations, sanitized Markdown renderer, repository permission checks, and Editorial repository workspace shell. It must not add GitHub visual styling, Primer/Octicons, JavaScript-side auth, a full wiki editor, or external GitHub API calls.

## Phase 1: Wiki Read Contract and Persistence - Home page data is screen-ready

**Done**: [x]

**Scope**: Add repository wiki persistence and a read-only API contract that can render `/wiki` as Home or a disabled/private/not-found state. This phase should create the database/API foundation without replacing the frontend placeholder yet.

**Key changes**:
- `crates/api/migrations/*_repository_wiki.*.sql`: add additive tables for `wiki_repositories`, `wiki_pages`, `wiki_page_revisions`, optional `wiki_page_toc_cache`, and indexes by repository/page slug/title/latest revision. Reuse `rendered_markdown_cache` rather than creating a second generic Markdown cache.
- `crates/api/src/domain/wiki.rs`: define DTOs such as `RepositoryWikiView`, `WikiPageView`, `WikiPageSummary`, `WikiRevisionSummary`, `WikiSidebarView`, `WikiFooterView`, `WikiViewer`, `WikiDisabledState`, and `WikiCloneInfo`.
- `crates/api/src/routes/repositories.rs` or a wiki route module: register `GET /api/repos/{owner}/{repo}/wiki` and `GET /api/repos/{owner}/{repo}/wiki/{slug}`. `/wiki` resolves `Home` first, then a deterministic first page only if no Home exists.
- Permission behavior: public repositories expose enabled wiki reads to anonymous readers; private repositories return repository-safe 404 for unauthorized viewers; disabled wikis return an explicit disabled state only when the repository itself is visible.
- Rendering behavior: render latest revision Markdown through the Rust sanitizer, generate heading anchors/outline, resolve `_Sidebar.*` and `_Footer.*`, and build clone URL metadata as `https://host/{owner}/{repo}.wiki.git`.
- `crates/api/tests/repository_wiki_contract.rs`: seed public/private repos, enabled/disabled wiki flags, Home and non-Home pages, sidebar/footer pages, revisions/authors, unsafe Markdown, and permissions; verify DTO shape, redirects/resolution, sanitized HTML, outline, clone URL, disabled/private states, and no session/OAuth/env leakage.
- `web/src/lib/api.ts` and server helpers: add typed wiki DTOs/fetchers with signed-cookie forwarding and no JS auth library.

**Verification**: focused Rust contract tests against `TEST_DATABASE_URL` when available, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, `cd web && npx tsc --noEmit --pretty false`, focused Biome for touched web types, then full `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Wiki Home Reader - repository tab, Markdown body, sidebar, and clone URL

**Done**: [x]

**Scope**: Replace the placeholder `/{owner}/{repo}/wiki` page with the Phase 1 Home reader inside the repository shell. The page should show the Wiki tab active, Markdown content, revision metadata, permissioned Edit/New Page buttons when available, page list, custom sidebar/footer, and a working clone URL copy control.

**Key changes**:
- `web/src/app/[owner]/[repo]/wiki/page.tsx`: server-fetch repository metadata and wiki view, render unavailable/disabled/private states, and preserve repository-safe not-found behavior.
- `web/src/components/RepositoryWikiPage.tsx`: render the Editorial wiki layout with a main Markdown article, title/revision line, author/timestamp metadata, Edit/New Page links only when `viewer.canEditWiki`, and a right column for pages, custom sidebar, footer, and clone URL.
- Reuse `MarkdownBody` for sanitized HTML only; do not bypass the Rust renderer or add a browser Markdown renderer.
- `web/src/components/WikiCloneCopyButton.tsx` or reuse `CopyButton`: implement clipboard copy with pending/success/error feedback and accessible label text.
- `web/src/lib/navigation.ts`: add wiki href helpers for Home and page slugs with safe encoding.
- `web/tests/repository-wiki-page.test.tsx`: cover active Wiki tab, Home title/body rendering, revision link, permissioned Edit/New Page visibility, page list links, custom sidebar/footer rendering, clone URL copy affordance, disabled state, no `href="#"`, no unsafe HTML, long title wrapping, mobile no-overflow, and Editorial banned-value guardrails.
- Browser smoke: load a seeded or stub-backed `/org/repo/wiki`, copy the clone URL, follow a page-list link target if Phase 3 route already exists or verify concrete hrefs only, check no placeholder controls, and save `ralph/screenshots/build/wiki-001-phase2-home.jpg`.

**Verification**: focused Vitest and smoke test, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Wiki Page Routes and Table of Contents - slug navigation and lazy outline expansion

**Done**: [x]

**Scope**: Add `/{owner}/{repo}/wiki/{slug}` reader behavior and make the right-column page list interactive. Clicking a page link should navigate to the slug route and update active state; expanding a page chevron should load or reveal that page's table of contents without dead controls.

**Key changes**:
- `web/src/app/[owner]/[repo]/wiki/[...slug]/page.tsx`: server-fetch the selected slug, render not-found/disabled/private states consistently, and keep the Wiki tab active.
- `GET /api/repos/{owner}/{repo}/wiki/{slug}/toc` or extend the existing page endpoint with bounded outline summaries for page-list expansion; prefer server-rendered outline data if cheap, otherwise a same-origin proxy for lazy loading.
- `RepositoryWikiPage`: highlight active page, preserve custom sidebar/footer, show current page title and revision metadata, and keep Home route canonical.
- `WikiPageList`: client component for chevron expansion, loading/error states, keyboard support, and nested outline links to heading anchors. The chevron must be a real button, not a static decoration.
- Slug normalization: support spaces and nested wiki paths through URL-safe slugs, reject traversal or empty slugs, and map unknown pages to an explicit missing-page state with permissioned New Page CTA.
- Tests: cover page slug routing, Home canonical behavior, active page state, chevron expansion, nested heading links, unknown page state, disabled/private behavior, slug encoding, no dead buttons, and mobile no-overflow.
- Browser smoke: navigate from `/wiki` to another page, expand that page's outline, click a heading anchor, and save `ralph/screenshots/build/wiki-001-phase3-page-navigation.jpg`.

**Verification**: focused Rust route/TOC contracts, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 4: Wiki States, Docs, Browser Evidence, and Build-Pass Bookkeeping

**Done**: [x]

**Scope**: Finish `wiki-001` only after the read contract, Home reader, slug reader, TOC expansion, clone copy, disabled/private states, docs, screenshots, and QA handoff are complete. Do not expand into wiki editing, page history diffs, full Git smart HTTP wiki transport, search indexing, or wiki settings mutations beyond reading the existing `wiki_enabled` flag.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document wiki read endpoints, Home resolution, slug behavior, TOC expansion, permissions, disabled/private states, sanitized Markdown, sidebar/footer resolution, clone URL semantics, and no-secret error envelopes.
- Final Rust tests: cover public/private repositories, disabled wiki flag, missing Home fallback, missing page, sidebar/footer precedence, revision author metadata, unsafe Markdown/links, outline generation, clone URL host derivation, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard traversal through page list/TOC/copy controls, revision links, custom sidebar/footer, empty wiki state, disabled private/visible distinctions, no placeholder hrefs, no inert click handlers, long Markdown/table/code content wrapping, and Editorial token compliance.
- `web/tests/e2e/repository-wiki.spec.ts`: signed-session sweep for Home read, slug navigation, TOC expansion, clone copy feedback, disabled wiki state when practical, screenshots, and mobile no-overflow.
- `qa-hints.json`: append targets for real wiki Git remote availability, large Markdown pages, duplicate title/slug normalization, stale rendered Markdown cache, unsafe links/images, private repository leakage, sidebar/footer edge cases, and keyboard navigation.
- `build-progress.txt`, `.qrspi/wiki-001/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `wiki-001.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/wiki-001-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, JSON validation for `prd.json` and `qa-hints.json`, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan:

```bash
rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'
```
