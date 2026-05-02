# Structure Outline: settings-001 Repository Settings Overview

**Ticket**: `settings-001`  
**Design**: `prd.json`, `BUILD_GUIDE.md`, `web/AGENTS.md`, `design/project/wf-settings.jsx`, existing repository shell/navigation, auth route guards, and repository data-access patterns.  
**Date**: 2026-05-03

## Phase 1: Admin-only settings read/write contract

**Done**: [x]

**Scope**: Add the persisted Rust API/data layer for repository Settings general controls. Only owner/admin users can read or mutate settings; non-admins receive structured forbidden envelopes.

**Key changes**:
- Add repository settings columns for Issues/Projects/Wiki, forking, and web commit signoff.
- Extend merge settings with auto-merge read state and enforce at least one enabled merge method.
- Add `repository_settings_audit_events` and record an event for every successful settings write.
- Add `GET/PATCH /api/repos/:owner/:repo/settings` with validation, forbidden, conflict, and database-safe error envelopes.
- Add focused Rust contract coverage for admin-only access, merge validation, persisted flags, and audit event count.

**Verification**: `CARGO_INCREMENTAL=0 cargo test -p opengithub-api --test api_repository_settings_contract`.

---

## Phase 2: Typed Next.js client and protected route surface

**Done**: [x]

**Scope**: Add typed API/client helpers and same-origin Next.js PATCH forwarding so the browser never calls the Rust API directly and signed cookies are preserved.

**Key changes**:
- Add `RepositorySettings` and update-request DTOs in `web/src/lib/api.ts`.
- Add server-session helper for repository settings reads.
- Add `web/src/app/[owner]/[repo]/settings/route.ts` to validate browser PATCH payloads and forward cookies to Rust.

**Verification**: `cd web && npx tsc --noEmit --pretty false`.

---

## Phase 3: Editorial Settings overview UI

**Done**: [x]

**Scope**: Replace the placeholder Settings page with a real two-column repository settings layout and working controls for general, feature, merge, forking, signoff, and danger-zone states.

**Key changes**:
- Keep repository workspace header with active Settings tab.
- Render grouped left settings navigation and right column card sections.
- Persist general settings via Save, and persist toggles only after the Rust API confirms.
- Validate merge-method changes before writes.
- Render destructive action typed-confirmation modals while leaving final destructive mutations disabled until backend support exists.
- Use Editorial tokens/classes and semantic CSS variables rather than GitHub/Primer colors.

**Verification**: `cd web && npm test -- --run tests/repository-settings-overview.test.tsx`; mandatory banned-value scan on touched UI files.

---

## Phase 4: Bookkeeping and final build handoff

**Done**: [x]

**Scope**: Record evidence and mark only `settings-001` build flags once the vertical slice passes focused and standard verification.

**Key changes**:
- Update `build-progress.txt` and `qa-hints.json` with concise settings-001 evidence and QA gaps.
- Update only `settings-001` in `prd.json`: `build_pass=true`, `needs_structure=false`, `qa_pass=false`.
- Commit and push `feat/opengithub-settings-001`; do not merge to main.

**Verification**: `make check`; `CARGO_INCREMENTAL=0 make test`; final git status excludes runtime junk, secrets, `target/`, and `node_modules/`.
