# Structure Outline: org-admin-005 Organization Member Privileges

**Ticket**: `org-admin-005`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-1.jsx`, `design/project/og-shell.jsx`, existing organization settings shell from `web/src/components/OrganizationSettingsShell.tsx`, organization policy defaults in `crates/api/migrations/202605040010_organization_create_flow.up.sql`, repository creation in `crates/api/src/domain/repositories.rs` and `web/src/app/new`, repository access settings in `web/src/components/RepositoryAccessSettingsPage.tsx`, repository Pages settings in `crates/api/src/domain/pages.rs`, and team creation policy checks in `crates/api/src/domain/organizations.rs`.
**Date**: 2026-05-04

## Phase 1: Owner Policy API Contract - read, patch, confirmation, and audit

**Done**: [x]

**Scope**: Add the authenticated Rust contract for organization member privilege settings. Owners can read and update `organization_policy_settings`; non-owners are denied; base repository permission and Projects base permission changes require explicit confirmation and return a refreshed policy state.

**Key changes**:
- Add only additive migrations if needed for `organization_policy_settings.enforced_by`, `organization_policy_settings.enforced_reason`, policy audit indexes, or `security_audit_events` links. Reuse existing columns for base repository permission, repository creation, private forking, discussions, Projects, Pages, app access requests, repository visibility/delete/transfer, issue deletion, and team creation.
- `crates/api/src/domain/organizations.rs`: add `OrganizationMemberPrivilegesSettings`, `OrganizationMemberPrivilegesPatch`, `OrganizationPolicyLock`, `OrganizationPolicyCapabilities`, `OrganizationPolicyAuditChange`, and `OrganizationMemberPrivilegesError`.
- Implement owner-only `organization_member_privileges_for_actor` and `update_organization_member_privileges_for_actor` with slug lookup, private-org outsider 404 privacy, non-owner 403, enum validation, confirmation tokens for base/projects permission downgrades or admin grants, partial patch behavior, before/after JSON values, and redacted `organization.policy.update` audit events.
- `crates/api/src/routes/organizations.rs`: add authenticated `GET /api/orgs/:org/settings/member-privileges` and `PATCH /api/orgs/:org/settings/member-privileges` with the standard error envelope.
- `crates/api/tests/organization_member_privileges_contract.rs`: cover anonymous 401, member/admin-but-not-owner 403, private outsider 404, owner read defaults, valid partial patch, invalid enum 422, required confirmation 409/422, audit before/after metadata, lock rendering fields, and no stack traces/secrets.

**Verification**: focused `organization_member_privileges_contract`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Member Privileges UI - long settings page with independent cards

**Done**: [x]

**Scope**: Build `/organizations/{org}/settings/member_privileges` on the existing Editorial organization settings shell. The page has independent cards/forms for base permissions, repository creation, private forking, discussions, Projects base permission, Pages publishing, app access requests, repository visibility/delete/transfer, issue deletion, and team creation.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed member-privileges DTOs, `getOrganizationMemberPrivilegesFromCookie`, and mutation helpers.
- `web/src/app/organizations/[org]/settings/member_privileges/page.tsx`: fetch session, app shell context, profile settings for the shell, and member privileges settings; render unavailable/forbidden states consistently with profile settings.
- `web/src/app/organizations/[org]/settings/member_privileges/actions/route.ts`: same-origin forwarding for per-card saves with the current session cookie and standard JSON envelopes.
- `web/src/components/OrganizationMemberPrivilegesPage.tsx`: render card-level forms using `.card`, `.btn`, `.chip`, `.input`, `.t-*`, and existing Editorial tokens only. Use action-menu/radio controls for None/Read/Write/Admin choices, checkboxes for repository creation/forking/discussions/Pages/destructive toggles, radio controls for app access requests, disabled policy-locked controls with a concrete why link, per-card pending/error/success state, and no inert controls.
- Base permission and Projects permission cards require an accessible confirmation dialog before submitting risky changes; failed server confirmation requirements should focus the dialog/error message.
- Add navigation entry in `web/src/lib/navigation.ts` for the member privileges settings section without breaking `/orgs/{org}/settings` compatibility.
- `web/tests/organization-member-privileges-page.test.tsx`: cover card rendering, default values, per-card payloads, confirmation dialog, disabled lock affordances, error/success feedback, no dead anchors/buttons, mobile wrapping classes, and Editorial banned-value/token expectations.
- `web/tests/e2e/organization-member-privileges.spec.ts`: owner smoke for initial render, base-permission confirmation, repository creation save, policy lock disabled state, team creation toggle, mobile no-overflow, and screenshot `ralph/screenshots/build/org-admin-005-phase2-member-privileges.jpg`.

**Verification**: focused Vitest and Playwright smoke, mandatory Editorial banned-value scan, then full `make check && make test`; run full `make test-e2e` when the local Playwright database/session setup is healthy.

---

## Phase 3: Repository Creation and Base Permission Enforcement

**Done**: [x]

**Scope**: Make policy settings affect repository creation and repository authorization. Organization repository creation honors public/private/internal creation toggles, and organization members receive the configured base repository permission while explicit direct/team grants continue to win when higher.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: enforce `members_can_create_public_repositories`, `members_can_create_private_repositories`, and `members_can_create_internal_repositories` in organization-owned create paths. Return clear policy-locked 403/422 envelopes with the organization policy reason and no private data leakage.
- Update repository owner/visibility option APIs used by `/new` so disabled organization visibility choices are omitted or rendered disabled with policy reasons instead of failing only after submit.
- Extend repository authorization helpers so `base_repository_permission` of `none/read/write/admin` applies to organization members, while explicit `repository_permissions` or team-derived permissions preserve the strongest permission.
- Update repository access settings display so inherited organization base permission rows and copy reflect the live policy value.
- `web/src/components/RepositoryCreatePage.tsx` and related tests: render constrained owner/visibility choices, disabled policy reasons, and server errors for stale policy changes without dead form states.
- Tests: extend Rust repository create/access contracts for public/private/internal creation denial/allowance, base none/read/write/admin authorization, explicit permission override, team permission override, audit presence from policy updates, and no leakage to non-members; extend Vitest/Playwright repository creation smoke for disabled organization visibility choices.

**Verification**: focused Rust contracts for organization member privileges plus repository create/access, focused web tests for repository creation/access display, focused Playwright create-flow smoke, then `make check && make test`; run full `make test-e2e` unless environment instability is documented and direct Playwright passes.

---

## Phase 4: Cross-Feature Policy Enforcement - teams, Pages, discussions, app access, and destructive toggles

**Done**: [x]

**Scope**: Enforce the remaining organization policy settings across existing API paths and UI affordances. Team creation respects the team-creation toggle, Pages publishing respects public/private Pages toggles, forking/discussions/project/app/destructive settings are represented in backend guards and rendered disabled where the underlying destructive feature is not fully implemented.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: update existing team creation checks to return the shared member-privileges policy envelope and audit linkage when `members_can_create_teams=false`.
- `crates/api/src/domain/pages.rs`: check organization policy before enabling public/private repository Pages publishing for organization-owned repositories; keep existing repo admin checks and return policy-locked errors when denied.
- Repository settings/update paths: enforce private forking, discussions, Projects base permission, repository visibility changes, repository deletion, repository transfer, and issue deletion toggles where those mutations already exist; for unsupported destructive flows, expose explicit disabled capability state rather than fake success.
- Add a small shared policy helper module or local helpers only if it reduces duplication between repositories, Pages, teams, and organizations without widening the architecture.
- Web surfaces touched by those policies: repository settings General, Pages, team creation, issue deletion affordances, and repository creation should show disabled controls or policy-lock messages that link back to member privileges for owners.
- Tests: extend Rust contracts for team creation denial, Pages public/private publishing denial, private forking/discussions/project capability states, visibility/delete/transfer/issue deletion disabled behavior, and audit/security audit metadata; extend focused Vitest/Playwright for disabled affordances and no dead controls.

**Verification**: focused Rust contracts for member privileges, teams, repository settings, Pages, and issues where applicable; focused web tests for policy-locked UI surfaces; focused Playwright smoke for member privileges plus one repository creation and one Pages denial path; then `make check && make test`.

---

## Phase 5: Docs, Browser QA Handoff, and Build Pass - finish org-admin-005

**Done**: [x]

**Scope**: Document organization member privilege endpoints, collect final browser evidence, update QA hints, and mark `org-admin-005.build_pass=true` only after every policy card and enforced API path is verified.

**Key changes**:
- `web/src/lib/api-docs.ts`: document `GET /api/orgs/{org}/settings/member-privileges` and `PATCH /api/orgs/{org}/settings/member-privileges`, including owner-only auth, field schema, confirmation behavior, policy locks, audit/security audit behavior, repository creation constraints, base permission enforcement, Pages/team creation/discussions/forking/destructive policy enforcement, and standard error envelopes.
- `web/tests/api-docs.test.tsx`: assert endpoint paths, request/response examples, confirmation and lock notes, audit notes, and cross-feature enforcement notes.
- `web/tests/e2e/organization-member-privileges.spec.ts`: final browser smoke for every visible card, all menus/radios/checkboxes, confirmation dialogs, per-card save success/error, policy-locked disabled controls, repository creation constraint, team creation constraint, Pages policy denial, docs link, mobile no-overflow, and no dead controls.
- Save screenshots in `ralph/screenshots/build/`: final long page, base permission confirmation, policy lock state, repository creation constrained state, Pages denial state, and mobile page.
- `qa-hints.json`: add honest QA focus areas for real Google owner sessions, concurrent policy writes, large organizations, base permission cache invalidation, policy races during repository creation/Pages deployment, destructive-toggle unsupported states, and screen-reader traversal of the long settings page.
- `build-progress.txt` and `prd.json`: record verification evidence and set `org-admin-005.build_pass=true`; leave `qa_pass=false`.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session DASHBOARD_E2E_SKIP_MIGRATIONS=1 make test-e2e`; browser smoke proves every member privileges card, menu, checkbox, radio, confirmation dialog, save button, lock link, repository creation constraint, team creation constraint, Pages policy denial, docs link, and mobile state has concrete behavior; mandatory Editorial banned-value scan returns zero matches.
