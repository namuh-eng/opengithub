# Structure Outline: settings-002 Repository Access Settings

**Ticket**: `settings-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-settings.jsx`, existing repository settings shell in `web/src/components/RepositorySettingsShell.tsx`, current access placeholder in `web/src/app/[owner]/[repo]/settings/access/page.tsx`, repository settings API patterns in `crates/api/src/domain/repositories.rs` / `crates/api/src/routes/repositories.rs`, permission roles in `crates/api/src/domain/permissions.rs`, organization/team data from `orgs-002`, and existing migrations for `repository_permissions`, `organization_memberships`, `team_memberships`, and `repository_settings_audit_events`.
**Date**: 2026-05-03

## Phase 1: Access API Contract - direct collaborators, teams, inherited access, and invitations

**Done**: [x]

**Scope**: Add the Rust/Postgres read and write contract for repository access management under `/api/repos/{owner}/{repo}/settings/access`. The contract must expose direct collaborators, team-derived access, owner/org inherited access, pending invitations, role definitions, available invite targets, and admin-only mutation capabilities without leaking private access data to non-admins.

**Key changes**:
- `crates/api/migrations/`: add additive access-management schema that is missing from the foundation, including `repository_invitations` with invite token hash, invitee email/user, role, status, expiry, actor, and indexes; add team-to-repository grant support if current `repository_permissions.source='team'` rows cannot represent team origin without losing `team_id`.
- `crates/api/src/domain/repositories.rs` or a focused `repository_access` domain module: add DTOs for `RepositoryAccessSettings`, `RepositoryAccessPerson`, `RepositoryAccessTeam`, `RepositoryInvitation`, `RepositoryRoleDefinition`, `RepositoryAccessSource`, `RepositoryAccessMutation`, and structured validation errors.
- `GET /api/repos/{owner}/{repo}/settings/access`: require a signed-in admin/owner; return 401 for anonymous, 403 for non-admin, 404 for missing repository, and no collaborator/team data for unauthorized private repositories.
- Include role hierarchy for `read`, `triage`, `write`, `maintain`, and `admin` in the API DTO even if lower-level app permissions still collapse to existing read/write/admin checks internally; preserve compatibility with `RepositoryRole` until broader permission phases expand.
- Compute access sources separately: `owner`, `direct`, `team`, `inherited`, and `pending`; inherited/team rows must include disabled mutation metadata and explanatory copy for the UI.
- Add mutation endpoints for invite user, invite team, change direct role, cancel pending invitation, and remove direct collaborator/team grant; successful mutations insert `repository_settings_audit_events`.
- Add `crates/api/tests/repository_access_settings_contract.rs` covering admin reads, anonymous/non-admin rejection, private repository privacy, direct collaborator rows, organization owner rows, team-derived rows, inherited disabled controls, pending invitations, role validation, invite target search bounds, audit events, and redacted errors.

**Verification**: focused `repository_access_settings_contract` against `TEST_DATABASE_URL`, then same-env `make check && make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Access Settings Shell - people, teams, sources, and pending invites

**Done**: [x]

**Scope**: Replace the `/[owner]/[repo]/settings/access` placeholder with a real Editorial Access page backed by the Phase 1 API. The page must keep the existing repository settings shell, show People and Teams sections or tabs, and render permission sources clearly with no GitHub/Primer visual regressions.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed access-settings DTOs and server fetch helpers that preserve forbidden/unavailable states.
- `web/src/app/[owner]/[repo]/settings/access/page.tsx`: fetch access data server-side and render the concrete access page; keep non-admin states explicit and non-leaky.
- `web/src/components/RepositoryAccessSettingsPage.tsx`: add People, Teams, and Pending invitations views with search/filter input, role chips/dropdowns, access source text, disabled inherited controls, owner/admin badges, and empty states with working CTAs.
- Use Editorial primitives and tokens only: `.card`, `.btn`, `.chip`, `.input`, `.av`, `.tabs`, `.list-row`, `.t-label`, `.t-mono-sm`, `var(--ink-*)`, `var(--line)`, `var(--accent)`, and semantic `.chip.ok/.warn/.err`.
- Rows must be keyboard focusable where interactive, use concrete links to user/team pages, and avoid `href="#"`, empty handlers, or display-only buttons.
- `web/tests/repository-access-settings-page.test.tsx`: cover admin rows, teams, inherited disabled state, pending invite rendering, forbidden/unavailable views, empty states, concrete controls, no inert anchors/buttons, and Editorial primitive/token usage.

**Verification**: focused Vitest for the access settings page, mandatory Editorial banned-value scan, then `make check && make test`. Save a browser screenshot if the E2E seed already exposes an admin repository.

---

## Phase 3: Access Mutations - invites, role changes, removals, and SES handoff

**Done**: [x]

**Scope**: Wire every visible access-management action to real API writes and confirmed server state. Admins must be able to invite users/teams, update direct roles, cancel pending invitations, and remove direct access after confirmation; inherited/team-derived rows must remain disabled with source-aware messaging.

**Key changes**:
- Add same-origin Next.js route handlers or server actions under the access settings route that forward authenticated mutation requests to the Rust API without adding JS-side auth.
- Implement Add person and Add team dialogs with search comboboxes, role selectors, validation, loading states, success/error feedback, and form reset after confirmed persistence.
- Implement role-change forms for direct collaborator/team grants with server-confirmed refresh; prevent changing owner or inherited/team-derived rows.
- Implement typed confirmation for removing direct access and canceling pending invitations; cancellation/removal must update the list only after the API returns fresh state.
- Add SES email handoff for user invitations through the configured AWS provider contract. If local SES credentials are absent, persist pending invitations and expose email delivery status/degraded message without marking the mutation as fake.
- Extend Rust tests for email enqueue/degraded paths, invite token expiry/status, duplicate invite conflicts, self-demotion protection for the last admin, organization membership/team validation, and audit-event integrity.
- Extend Vitest coverage for dialogs, combobox filtering, validation errors, success notices, role changes, cancellation/removal confirmations, inherited disabled controls, and no local-only state changes.
- Add `web/tests/e2e/repository-settings-access.spec.ts`: seed an admin repository, invite/cancel a user, change a direct role, remove a direct collaborator, verify persisted reload state, and save `ralph/screenshots/build/settings-002-phase3-access-mutations.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` when local database/dev servers are stable.

---

## Phase 4: Privacy, Role Semantics, and Team/Organization Guardrails

**Done**: [x]

**Scope**: Harden repository access management around authorization, role hierarchy, organization/team inheritance, pending invitation lifecycle, long content, and responsive/accessibility behavior. This phase should complete the PRD behavior without broadening into branch rules or personal access-token settings.

**Key changes**:
- Rust guardrails: final coverage for public/private/internal repositories, user-owned versus organization-owned repositories, org base permission rows, team membership changes, team grant removal, pending invitation cancellation by non-creator admins, duplicate team/user grants, unknown users/teams, and structured 400/401/403/404/409 errors.
- Protect against privilege mistakes: disallow removing the last admin/owner path, disallow demoting owner-derived access, disallow team mutations for non-org repositories, and make inherited rows read-only.
- Frontend guardrails: long usernames/team names/emails wrap cleanly, role dropdowns fit mobile widths, disabled controls expose accessible explanations, confirmation dialogs trap focus, and empty states have working Add buttons.
- API docs: document `GET /api/repos/{owner}/{repo}/settings/access` and access mutation endpoints in `web/src/lib/api-docs.ts` with permission, role, invitation, audit-event, and SES/degraded-delivery notes.
- E2E: browser smoke for admin access page, forbidden non-admin state, mobile no-overflow, keyboard navigation through dialogs, and screenshots under `ralph/screenshots/build/settings-002-phase4-*.jpg`.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make test`; run full `make test-e2e` when the local DB and dev servers are healthy.

---

## Phase 5: Final QA Handoff and Build-Pass Bookkeeping - finish settings-002

**Done**: [ ]

**Scope**: Lock repository Access settings as a completed vertical slice and mark `settings-002.build_pass=true` only after API contract, UI, mutation flows, SES/degraded delivery behavior, audit events, admin-only privacy, browser smoke, docs, and QA handoff are verified. This phase should not add branches, webhooks, Pages, secrets, or security subfeatures.

**Key changes**:
- Finalize `repository_access_settings_contract`, access settings Vitest, API docs tests, and Playwright coverage for the complete Access page.
- Ensure every visible button/link/form has concrete behavior or a disabled unsupported/inherited state with accessible text.
- Ensure all direct access writes create audit events and return fresh server state after mutation.
- Add/extend `qa-hints.json` with deeper QA targets: real SES email delivery, invite acceptance lifecycle, concurrent role edits, last-admin protection, organization base permissions, team membership changes after grants, pending invite expiry, and private repository leakage checks.
- Update `build-progress.txt`, `.qrspi/settings-002/structure.md`, and `prd.json`; set `settings-002.build_pass=true` only after final verification; leave `qa_pass=false`.
- Run the mandatory Editorial banned-value scan before commit and fix any touched-file regressions.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when available, focused browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
