# profiles-001 Vertical Structure

## Goal
Render a real public user profile overview at `/<user>` with identity card, follow/block/report controls, URL tabs, pinned items, achievements, and an accessible contribution graph using the OpenGitHub Editorial design system.

## Phase 1 — Profile data contract
- Add database-backed profile tables/columns for public profile metadata, profile README, pins, contribution days/events, achievements, blocks, and reports.
- Add Rust domain/service functions for profile lookup by login, viewer relationship state, pins, achievements, and contributions.
- Add REST endpoints:
  - `GET /api/users/:login/profile`
  - `PUT|DELETE /api/users/:login/follow`
  - `PUT|DELETE /api/users/:login/block`
  - `POST /api/users/:login/report`

## Phase 2 — Next.js data bridge
- Add typed API shapes and cookie-forwarding helpers in `web/src/lib/api.ts`.
- Add server-session helper for profile reads.
- Add Next route handlers under `web/src/app/[owner]/actions/*` for follow/block/report mutations so client controls use same-origin endpoints and preserve cookies.

## Phase 3 — Editorial profile UI
- Replace the placeholder `web/src/app/[owner]/page.tsx` with a profile overview page.
- Add reusable profile components for the identity card, tabs, pinned grid, achievements, private profile state, controls, and accessible contribution graph.
- Use only `og.css` / `og-themes.css` tokens and primitives; no Primer, Octicons, GitHub palette, or inert controls.

## Phase 4 — Tests and verification
- Add Rust API contract tests for anonymous reads, signed-in follow/block/report writes, private profile hiding, and no sensitive leakage.
- Add React/Vitest coverage for profile rendering, controls, private mode, and accessible contribution labels.
- Run focused tests, banned-value scan, `make check`, and `make test` before marking build complete.

## Phase 5 — Build metadata
- Update `build-progress.txt`, `qa-hints.json`, and `prd.json` evidence for `profiles-001`.
- Set `profiles-001.build_pass=true` only after acceptance coverage plus `make check` and `make test` pass; leave `qa_pass=false` for independent QA.

## Completion evidence (2026-05-02)
- Phase 1 complete: migration `202605020038_user_profiles`, Rust profile domain, and API routes added.
- Phase 2 complete: typed web API helpers, server-session bridge, and same-origin action routes added.
- Phase 3 complete: `/{user}` renders the Editorial profile overview with real controls and private-profile handling.
- Phase 4 complete: focused Rust/Vitest tests, banned-value scan, `make check`, and `make test` passed.
- Phase 5 complete: `build-progress.txt`, `qa-hints.json`, and `prd.json` updated for build handoff.
