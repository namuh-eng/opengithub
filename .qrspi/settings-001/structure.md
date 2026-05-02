# Structure Outline: settings-001 Repository General Settings

**Ticket**: `settings-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-settings.jsx`, current repository settings placeholders in `web/src/components/RepositorySettingsShell.tsx` / `RepositorySettingsSectionPage.tsx`, current repository API in `crates/api/src/routes/repositories.rs`, current repository domain in `crates/api/src/domain/repositories.rs`, existing `repository_merge_settings` migration, and existing navigation tests.
**Date**: 2026-05-03

## Phase 1: Repository Settings API Contract - admin-only read/write and audit events

**Done**: [x]

**Scope**: Add the Rust/Postgres contract for `GET /api/repos/{owner}/{repo}/settings` and `PATCH /api/repos/{owner}/{repo}/settings`. The endpoint must only load or mutate for viewers with admin/owner repository permission, return a dense settings DTO for the General settings page, validate every mutation, persist audit events for successful writes, and return structured errors for forbidden, validation, not-found, conflict, and database states.

**Key changes**:
- `crates/api/migrations/`: add additive repository settings fields that are not already present on `repositories` or `repository_merge_settings`, including issue/project/wiki feature flags, allow forking, web commit signoff requirement, and `repository_settings_audit_events`.
- `crates/api/src/domain/repositories.rs`: add `RepositorySettings`, `RepositorySettingsPatch`, `RepositoryFeatureSettings`, `RepositoryMergeSettings`, `RepositoryDangerState`, and `RepositorySettingsAuditEvent` DTOs plus helpers to read settings, validate patch payloads, apply partial updates transactionally, and insert audit rows.
- Reuse existing `repositories.visibility`, `repositories.default_branch`, `repositories.is_archived`, `repositories.is_template`, and `repository_merge_settings` instead of duplicating state.
- Validation rules: repository name must keep the existing format/uniqueness rules; visibility must be `public`, `private`, or `internal`; default branch must exist in `repository_git_refs`; at least one merge method must remain enabled; archived repositories reject non-archive mutations except unarchive; destructive delete/transfer actions remain unsupported and disabled in the response.
- `crates/api/src/routes/repositories.rs`: add routes under `/:owner/:repo/settings` with optional typed field-level errors and consistent JSON envelopes.
- `crates/api/tests/repository_settings_contract.rs`: cover admin read/write, non-admin 403, anonymous 401, private repository behavior, merge-method validation, branch validation, feature flag persistence, rename uniqueness conflicts, audit-event creation, and redacted errors.

**Verification**: focused `repository_settings_contract` against `TEST_DATABASE_URL`, then same-env `make check && make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial General Settings Shell - render real repository settings

**Done**: [ ]

**Scope**: Replace the `/[owner]/[repo]/settings` placeholder with a real Editorial General settings page backed by the Phase 1 API. The page must keep the repository workspace header and settings sidebar, show the current repository state, and avoid GitHub/Primer visual values while preserving the same information architecture.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed settings DTOs and server fetch helpers for the repository settings read contract.
- `web/src/app/[owner]/[repo]/settings/page.tsx`: fetch settings server-side and render a concrete page for admins; render unavailable/forbidden states without leaking settings to non-admins.
- `web/src/components/RepositoryGeneralSettingsPage.tsx`: add card sections for repository name, description/social preview, visibility, template flag, feature toggles, pull request merge methods, default branch, forking, web commit signoff, archive state, and danger zone.
- Use existing primitives and tokens: `.card`, `.btn`, `.chip`, `.input`, `.t-label`, `.t-body`, `var(--ink-*)`, `var(--accent)`, `var(--line)`, and semantic `.chip.ok/.warn/.err`.
- All unsupported destructive controls must be disabled with explicit unavailable state and no dead handlers; no `href="#"`, no empty buttons, and no optimistic local-only state.
- `web/tests/repository-general-settings-page.test.tsx`: cover rendering, admin-only state, default values, disabled danger-zone actions, no inert anchors/buttons, and Editorial token/primitives expectations.

**Verification**: focused Vitest for the settings page, mandatory Editorial banned-value scan, then `make check && make test`. Save a browser screenshot if the E2E seed already exposes an admin repository.

---

## Phase 3: Settings Mutations - forms, validation feedback, and confirmed writes

**Done**: [ ]

**Scope**: Make every editable General settings control submit to the Rust API and update UI only after confirmed persistence. Each section can have its own save action, but every control must map to the shared patch contract and show success/error feedback without dead `onClick` placeholders.

**Key changes**:
- Add same-origin Next.js route handlers or server actions under the repository settings route that forward authenticated requests to the Rust API without introducing JS-side auth.
- Implement forms for repository name/description, visibility/template/default branch, feature toggles, merge methods/default merge method, forking, web commit signoff, and archive/unarchive.
- Merge settings must disable submission or return inline errors when all merge methods are off, and the API must remain authoritative.
- Feature toggles and checkboxes must preserve current server state after failed writes and show field-level errors from Rust.
- `web/tests/repository-general-settings-page.test.tsx`: extend for form names, submit buttons, inline errors, success notices, disabled invalid merge state, and no local-only state changes.
- `web/tests/e2e/repository-settings-general.spec.ts`: seed an admin repository, change description/features/merge settings, verify persisted reload state, verify validation errors, and save `ralph/screenshots/build/settings-001-phase3-general-mutations.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke, then `make check && make test`; run `make test-e2e` when local database/dev servers are stable.

---

## Phase 4: Admin Privacy, Conflict, and Danger-Zone Guardrails

**Done**: [ ]

**Scope**: Harden repository settings against unauthorized access, concurrent/conflicting writes, unsupported destructive actions, archive behavior, and UI regressions. This phase should complete behavior required by the PRD without adding repository access/branches/webhooks/secrets subfeatures.

**Key changes**:
- Rust tests: final coverage for anonymous/non-admin/admin/owner users, organization-owned repositories, private/internal repositories, archived repository mutation restrictions, rename conflicts across user/org owners, default-branch deletion or missing branch conflicts, and redacted structured errors.
- Frontend tests: verify forbidden/unavailable states, disabled delete/transfer controls with typed-confirmation modal affordances, archive confirmation, long repository names/descriptions, mobile wrapping, keyboard focus, and no dead controls.
- E2E: admin smoke for load, save, failed validation, reload persistence, archive/unarchive confirmation, and non-admin forbidden view if the seed supports it; save desktop and mobile screenshots under `ralph/screenshots/build/`.
- `web/src/lib/api-docs.ts`: document `GET /api/repos/{owner}/{repo}/settings` and `PATCH /api/repos/{owner}/{repo}/settings` with permission, validation, and audit-event notes.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make test`; run full `make test-e2e` when the local DB and dev servers are healthy.

---

## Phase 5: Final QA Handoff and Build-Pass Bookkeeping - finish settings-001

**Done**: [ ]

**Scope**: Lock the General repository settings feature as a completed vertical slice and mark `settings-001.build_pass=true` only after API contract, UI, mutation flows, audit events, admin-only access, browser smoke, docs, and QA handoff are verified. This phase should not add new settings sections.

**Key changes**:
- Finalize `repository_settings_contract`, repository settings Vitest, API docs tests, and Playwright coverage for the complete General page.
- Ensure every visible button/link/form has concrete behavior or a disabled unsupported state with accessible text.
- Ensure all settings writes create audit events and return fresh server state after mutation.
- Add/extend `qa-hints.json` with deeper QA targets: concurrent settings writes, long repository names/descriptions, private/internal visibility transitions, archived repository behavior, unsupported delete/transfer controls, and audit-event integrity.
- Update `build-progress.txt`, `.qrspi/settings-001/structure.md`, and `prd.json`; set `settings-001.build_pass=true` only after final verification; leave `qa_pass=false`.
- Run the mandatory Editorial banned-value scan before commit and fix any touched-file regressions.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when available, focused browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
