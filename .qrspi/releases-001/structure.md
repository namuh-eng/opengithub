# releases-001 structure

## Goal
Render repository releases list, latest release detail, release reactions, assets, and paginated release history for `/{owner}/{repo}/releases` and `/{owner}/{repo}/releases/latest` using OpenGitHub Editorial UI primitives and a typed deterministic data/API layer.

## Constraints
- Work only in this isolated worktree on `feat/opengithub-releases-001`.
- Clone GitHub capabilities, not GitHub visuals; use `web/src/app/og.css` / `og-themes.css` tokens and primitives.
- Prefer real backend/API/data behavior; if DB tables are not present, use typed deterministic server-side data with an explicit upgrade seam.
- Reaction controls must reflect auth capability: toggle only via API path when authenticated; public/unauthenticated users see disabled/sign-in state.
- Keep `qa_pass=false`; set `build_pass=true` only after acceptance is complete and `make check` + `make test` pass.

## Vertical phases

### Phase 1 — Discovery and contracts
- Confirm current repository shell/navigation patterns, existing release placeholders, tests, and metadata files.
- Define typed release domain objects covering release author, tag/commit, labels, Markdown notes, contributors, assets, archives, reactions, pagination, and permissions.
- Acceptance: data contract can answer list page, latest non-prerelease detail, specific release lookup, and reaction state transitions deterministically.

### Phase 2 — Server/API slice
- Add a deterministic server-side release repository module with seeded data and helper functions.
- Add route handlers for release history/latest and release reaction toggles, with public read and auth-gated writes.
- Acceptance: focused tests verify ordering, pagination, latest resolution, asset/archive presence, and reaction auth/toggle semantics.

### Phase 3 — UI slice
- Replace placeholder release page with a repository workspace release experience: Releases/Tags subnav, release cards, Markdown notes, contributor avatars, assets disclosure, archives, reaction bar, compare navigation, and pagination.
- Add `/releases/latest` focused detail route resolving to the latest non-prerelease release.
- Acceptance: UI uses Editorial classes/tokens and no banned GitHub/Primer/Octicon values.

### Phase 4 — Verification and metadata
- Add focused component/API tests for releases.
- Run mandatory banned-value scan, focused tests, `make check`, and `make test`.
- Update `build-progress.txt`, `qa-hints.json`, and `prd.json` with concise evidence.
- Commit with Lore-compatible message and push branch.
