# Structure Outline: org-admin-003 Organization People Administration

**Ticket**: `org-admin-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-1.jsx`, `design/project/og-shell.jsx`, existing public people page in `web/src/components/OrganizationPeoplePage.tsx`, existing organization settings shell from `.qrspi/org-admin-002/structure.md`, current organization people API in `crates/api/src/domain/organizations.rs` and `crates/api/src/routes/organizations.rs`, existing repository invitation patterns in `crates/api/src/domain/repositories.rs` and `web/src/components/RepositoryAccessSettingsPage.tsx`, and the organization people contract in `crates/api/tests/organization_people_contract.rs`.
**Date**: 2026-05-04

## Phase 1: Owner People Admin API Contract - list members, tabs, and mutation state

**Done**: [x]

**Scope**: Add the authenticated owner/admin organization people management contract while preserving the current public `/api/orgs/{org}/people` read behavior. Owners/admins can load Members, Outside collaborators, Pending collaborators, Invitations, Failed invitations, and Security Managers with search, counts, row capabilities, 2FA/session signals, role/source metadata, and export URLs. Non-admin members and outsiders must not see admin-only details.

**Key changes**:
- `crates/api/migrations/<timestamp>_organization_people_admin.*.sql`: add only missing additive storage for organization invitations, invitation email delivery status, membership public/private visibility, optional outside-collaborator markers, failed invitation metadata, and indexes for organization/member/invitation lookup. Reuse `organization_memberships`, `teams`, `team_memberships`, `users`, `user_email_addresses`, `organization_audit_events`, and notification/email delivery tables where they already exist.
- `crates/api/src/domain/organizations.rs`: add `OrganizationPeopleAdmin`, `OrganizationPeopleAdminTab`, `OrganizationPeopleAdminRow`, `OrganizationInvitationRow`, `OrganizationPeopleAdminQuery`, `OrganizationPeopleAdminFilters`, `OrganizationPeopleAdminExport`, `OrganizationPeopleAdminActionState`, and `OrganizationPeopleAdminError`.
- Admin read rules: owners/admins can see all member roles, 2FA/session-derived status, private membership state, pending/failed invitations, export affordances, team/role counts, membership source, and row action capabilities; non-admins receive `403`, outsiders on private organizations receive `404`, and all errors use the standard envelope without leaking private emails or invite tokens.
- `crates/api/src/routes/organizations.rs`: add authenticated `GET /api/orgs/:org/people/admin` with `tab`, `q`, `page`, `pageSize`, and `format`-ready filter normalization. Keep public `GET /api/orgs/:org/people` compatible.
- `crates/api/tests/organization_people_admin_contract.rs`: cover owner read, admin read, member 403, outsider/private 404, tab counts, search/pagination, hidden invite-token hashes, public/private membership flags, 2FA/session status flags, role/source metadata, final-owner capability flags, and no raw stack traces/secrets.

**Verification**: focused `organization_people_admin_contract`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial People Management Shell - render tabs, filters, rows, and disabled-safe controls

**Done**: [ ]

**Scope**: Replace the read-only public people presentation for owner/admin viewers with an Editorial organization people administration surface at `/orgs/{org}/people`. The page must keep the public/member read behavior for non-admin viewers, but owners/admins see the GitHub-like people management information architecture rendered with the Editorial design system.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed admin people DTOs and `getOrganizationPeopleAdminFromCookie`.
- `web/src/components/OrganizationPeopleAdminPage.tsx`: add the admin shell with tabs for Members, Outside collaborators, Pending collaborators, Invitations, Failed invitations, and Security Managers; a toolbar with member search, disabled bulk action until rows are selected, export menu, and Invite member CTA.
- Row UI: checkbox, avatar, display name, username, 2FA chip, membership visibility menu, role chip, teams count, roles count, action menu, membership source, invitation status, and retry/cancel affordances as applicable. Use `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.t-*` classes and tokenized colors only.
- `web/src/app/orgs/[org]/people/page.tsx`: fetch both public profile data and the admin people contract when the viewer is signed in; render admin state only when `viewerState.canAdmin` or the admin endpoint succeeds.
- Export controls may render concrete same-origin URLs and disabled unavailable states in this phase, but no dead `href="#"` or empty handlers are allowed.
- `web/tests/organization-people-admin-page.test.tsx`: cover tab links, search form, selected-row bulk enablement, export dropdown behavior, disabled security-manager/out-of-scope states, row menus, empty states, non-admin fallback, no dead anchors/buttons, no secret/email-token rendering, and Editorial token expectations.
- `web/tests/e2e/organization-people-admin.spec.ts`: focused owner smoke for opening `/orgs/{org}/people`, switching tabs, searching, opening menus/dropdowns, selection enabling bulk controls, and saving `ralph/screenshots/build/org-admin-003-phase2-people-shell.jpg`.

**Verification**: focused Vitest and Playwright smoke, mandatory Editorial banned-value scan, then full `make check && make test`; run `make test-e2e` when the local database and auth seed are stable.

---

## Phase 3: Invitations and Email Delivery - invite, retry, and cancel pending people

**Done**: [ ]

**Scope**: Wire the Invite member dialog and pending/failed invitation actions to real Rust endpoints. Owners/admins can invite by username or verified email, choose role/team defaults, send or degrade the SES handoff honestly, retry failed deliveries, and cancel pending invitations. Invitations expire after 7 days.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: add `CreateOrganizationInvitation`, `OrganizationInvitationMutationResult`, invitation target resolution for username/email, role/team validation, 7-day expiry calculation, duplicate-pending detection, invite token hashing, SES/email-delivery record insertion, retry state transitions, cancel state transitions, and redacted audit events.
- `crates/api/src/routes/organizations.rs`: add `POST /api/orgs/:org/people/invitations`, `POST /api/orgs/:org/people/invitations/:invitation_id/retry`, and `DELETE /api/orgs/:org/people/invitations/:invitation_id`, returning fresh admin people state after each write.
- Do not fake external mail success: when SES/local email credentials are missing or delivery cannot be confirmed, persist a degraded/failed delivery state and expose retry affordances.
- `web/src/app/orgs/[org]/people/actions/route.ts`: add same-origin forwarding for invite, retry, and cancel actions with the current cookie and standard JSON envelopes.
- `web/src/components/OrganizationPeopleAdminPage.tsx`: add the Invite member dialog with username/email autocomplete from available user/team targets, role/team choices, send pending state, field errors, success feedback, retry/cancel confirmations, and no duplicate-submit behavior.
- Tests: extend Rust contract for successful invite, duplicate invite conflict, existing member conflict, invalid role/team, SES degraded state, retry, cancel, expiry, audit redaction, and token-hash redaction; extend Vitest/Playwright for dialog validation, retry/cancel, failed tab recovery, and screenshot `ralph/screenshots/build/org-admin-003-phase3-invitations.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright invite/retry/cancel smoke with fresh DB seed, then `make check && make test`; run full `make test-e2e` if the Playwright auth/database setup is healthy.

---

## Phase 4: Membership Mutations and Exports - visibility, roles, removal, and downloads

**Done**: [ ]

**Scope**: Complete owner/admin people management mutations and exports. Visibility changes, role changes, member removal, bulk action enablement, and JSON/CSV exports must hit real endpoints with final-owner protections and deterministic error feedback.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: add `UpdateOrganizationMembershipVisibility`, `UpdateOrganizationMembershipRole`, `RemoveOrganizationMember`, `OrganizationPeopleExportFormat`, final-owner guard helpers, membership mutation audit rows, and filtered member export builders for JSON and CSV.
- `crates/api/src/routes/organizations.rs`: add `PATCH /api/orgs/:org/people/members/:user_id/visibility`, `PATCH /api/orgs/:org/people/members/:user_id/role`, `DELETE /api/orgs/:org/people/members/:user_id`, and `GET /api/orgs/:org/people/export?format=json|csv&tab=...&q=...`.
- Mutation rules: final owner cannot be demoted or removed; owners cannot remove themselves when they are the final owner; invalid role transitions return structured `422`; non-owner/admin callers receive `403`; private members and private emails never leak through exports to unauthorized callers.
- `web/src/components/OrganizationPeopleAdminPage.tsx`: wire visibility menu, role-change confirmation, remove confirmation, selected-row bulk action placeholder with explicit unsupported state if bulk mutation execution is deferred, export JSON/CSV download links, success/error feedback, focus return, mobile wrapping, and no dead controls.
- Tests: extend Rust contract for public/private visibility persistence, role changes, final-owner demotion/removal blocks, successful removal, filtered CSV/JSON exports, unauthorized export denial, audit metadata, and no secret leakage; extend Vitest/Playwright for menus, confirmations, export URLs/download response, final-owner errors, mobile no-overflow, and screenshot `ralph/screenshots/build/org-admin-003-phase4-member-actions.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright desktop/mobile mutation/export smoke, then `make check && make test`; run full `make test-e2e` unless local environment instability is documented.

---

## Phase 5: Docs, Browser QA Handoff, and Build Pass - finish org-admin-003

**Done**: [ ]

**Scope**: Document the organization people administration API and complete final build-loop bookkeeping. Mark `org-admin-003.build_pass=true` only after admin listing, invitations, visibility/role/removal mutations, exports, browser smoke, and QA handoff are verified.

**Key changes**:
- `web/src/lib/api-docs.ts`: document the organization people admin endpoints, including auth/owner requirements, tab/filter shape, invitation lifecycle, SES degraded delivery behavior, membership visibility and role mutation rules, final-owner protections, exports, audit behavior, and token/secret redaction.
- `web/tests/api-docs.test.tsx`: assert organization people admin endpoint paths, request/response shapes, standard errors, final-owner notes, email delivery notes, and export notes render without raw stack trace or secret examples.
- `web/tests/e2e/organization-people-admin.spec.ts`: final smoke for admin tabs, search, row selection, invite success/error, retry/cancel pending invitation, visibility toggle, role confirmation, final-owner block, member removal, JSON/CSV export, mobile no-overflow, and no dead controls.
- `ralph/screenshots/build/`: save final desktop members table, invite dialog, pending/failed tabs, role/remove confirmation, export menu, and mobile screenshots when test data allows.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `org-admin-003.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session DASHBOARD_E2E_SKIP_MIGRATIONS=1 make test-e2e`; browser smoke proves every visible organization people tab, filter, row, checkbox, dropdown, dialog, form, export link, empty-state CTA, error state, and redirect has concrete behavior; mandatory Editorial banned-value scan returns zero matches.
