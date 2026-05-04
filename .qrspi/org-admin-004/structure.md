# Structure Outline: org-admin-004 Organization Teams

**Ticket**: `org-admin-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-1.jsx`, `design/project/og-shell.jsx`, existing organization profile/people surfaces from `.qrspi/org-admin-002/structure.md` and `.qrspi/org-admin-003/structure.md`, current team storage in `teams`, `team_memberships`, `team_repository_permissions`, repository access settings in `crates/api/src/domain/repositories.rs` and `web/src/components/RepositoryAccessSettingsPage.tsx`, organization people administration in `crates/api/src/domain/organizations.rs`, notification fanout in `crates/api/src/domain/notifications.rs`, and app-shell team links in `crates/api/src/domain/app_shell.rs`.
**Date**: 2026-05-04

## Phase 1: Teams Directory API Contract - list, visibility, search, and empty state data

**Done**: [ ]

**Scope**: Add the authenticated Rust contract for `/api/orgs/{org}/teams` while preserving existing concrete team detail URLs. Organization members can list visible teams, owners/admins can see secret teams and management affordances, and empty organizations receive the three-column explanatory empty-state payload required by the PRD.

**Key changes**:
- `crates/api/migrations/<timestamp>_organization_teams_admin.*.sql`: add only missing additive columns/indexes for `teams.parent_team_id`, `teams.visibility`, `teams.notifications_enabled`, slug uniqueness per organization, parent lookup, optional team mention index storage, and audit indexes. Reuse `teams`, `team_memberships`, `team_repository_permissions`, `organization_memberships`, `repositories`, `repository_permissions`, `notification_subscriptions`, and `organization_audit_events`.
- `crates/api/src/domain/organizations.rs`: add `OrganizationTeamsDirectory`, `OrganizationTeamSummary`, `OrganizationTeamParentOption`, `OrganizationTeamsQuery`, `OrganizationTeamCapabilities`, and `OrganizationTeamsError`.
- Directory rules: visible teams are discoverable and mentionable by organization members; secret teams appear only to team members and organization owners/admins; outsiders on private orgs receive `404`; members without admin rights receive read-only capability flags.
- `crates/api/src/routes/organizations.rs`: add authenticated `GET /api/orgs/:org/teams` with `q`, `visibility`, `page`, and `pageSize` normalization and the standard list envelope.
- `crates/api/tests/organization_teams_contract.rs`: cover anonymous 401, private-org outsider 404, member visible-team list, secret-team privacy, owner/admin full list, filtering/pagination, parent summary fields, repository/member counts, empty-state data, capability flags, and no raw invite/member private data leakage.

**Verification**: focused `organization_teams_contract`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Teams Directory UI - list, filters, empty state, and concrete navigation

**Done**: [ ]

**Scope**: Replace the teams placeholder with a real Editorial organization teams directory at `/orgs/{org}/teams`. The page renders the empty state with flexible repository access, request-to-join teams, and team mentions columns, plus working New team and Learn more CTAs. Populated lists filter teams and link rows to `/orgs/{org}/teams/{teamSlug}`.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed teams-directory DTOs and `getOrganizationTeamsFromCookie`.
- `web/src/app/orgs/[org]/teams/page.tsx`: fetch organization shell/session context and render the teams directory or unavailable/forbidden states.
- `web/src/components/OrganizationTeamsPage.tsx`: add Editorial header, tabs/filter form, team rows with avatar/slug/visibility/member/repository counts, parent breadcrumbs, mentionability chips, and the required three-column empty state. Use `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, and `.t-*` primitives only.
- Navigation: row links, New team CTA to `/orgs/{org}/teams/new`, Learn more to `/docs/api#organization-teams` or the closest concrete docs route, no `href="#"`, no empty handlers, and mobile wrapping for long team names/slugs.
- `web/tests/organization-teams-page.test.tsx`: cover empty state columns/CTAs, populated rows, search/filter form, secret/visible chips, parent labels, forbidden/unavailable states, no dead anchors/buttons, no secret data for member-only fixtures, and Editorial token/primitives expectations.
- `web/tests/e2e/organization-teams.spec.ts`: focused smoke for owner and member list views, search/filter behavior, row navigation to an existing team detail URL, empty state CTAs, mobile no-overflow, and screenshot `ralph/screenshots/build/org-admin-004-phase2-teams-directory.jpg`.

**Verification**: focused Vitest and Playwright smoke, mandatory Editorial banned-value scan, then full `make check && make test`; run full `make test-e2e` when the local Playwright database/session setup is healthy.

---

## Phase 3: Team Creation and Nesting Rules - create visible or secret teams safely

**Done**: [ ]

**Scope**: Wire `/orgs/{org}/teams/new` to a real Rust creation endpoint. Owners/admins, and members only when organization policy allows, can create teams with slugified unique names, description, optional visible parent team, visibility, and notification preference. Secret teams cannot be nested, and no team may create a parent cycle.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: add `CreateOrganizationTeam`, `OrganizationTeamCreateResult`, slug normalization helpers, duplicate-slug conflict handling, parent-team validation, secret-nesting rejection, cycle guard helpers, notification default persistence, and redacted `organization.team.create` audit events.
- `crates/api/src/routes/organizations.rs`: add authenticated `POST /api/orgs/:org/teams`, returning the created team summary plus destination href.
- `web/src/app/orgs/[org]/teams/new/page.tsx`: add the protected creation page with server-fetched parent options and organization policy state.
- `web/src/app/orgs/[org]/teams/actions/route.ts`: add same-origin forwarding for team creation with the current cookie and standard JSON envelopes.
- `web/src/components/OrganizationTeamCreatePage.tsx`: add the Team name, Description, Parent team selector or "no teams" text, Visible/Secret radio group, Enabled/Disabled notification radio group, Create team button, field-level validation, pending state, success redirect, and non-dead Cancel/Learn more links.
- Tests: extend Rust contract for create success, slugification, duplicate conflict, member-policy denial/allowance, invalid parent, secret parent/child block, cycle guard, audit row, and notification flag; extend Vitest/Playwright for validation, radio states, no-parent copy, duplicate error, successful redirect, and screenshot `ralph/screenshots/build/org-admin-004-phase3-team-create.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright create-flow smoke with fresh DB seed, then `make check && make test`; run full `make test-e2e` unless local environment instability is documented.

---

## Phase 4: Team Detail Foundation, Permissions, Mentions, and Notifications

**Done**: [ ]

**Scope**: Make team rows land on useful team detail pages and connect team permissions to repository authorization/review/notification foundations. Team detail shows members, repositories, parent/child inheritance, mentionability, and notification state; repository authorization and review request lookups honor direct and inherited team permissions.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: add `OrganizationTeamDetail`, `OrganizationTeamRepositoryPermission`, `OrganizationTeamMemberRow`, `OrganizationTeamHierarchy`, inherited permission resolution helpers, and mention target lookup for visible/member-authorized secret teams.
- `crates/api/src/routes/organizations.rs`: add authenticated `GET /api/orgs/:org/teams/:team_slug`, plus narrow mutation endpoints only where needed for notification preference or repository-permission refresh if existing repository access endpoints cannot cover the flow.
- Permission rules: parent repository permissions cascade to children for authorization and review requests; secret child restrictions stay enforced; deleting or changing team grants refreshes team-derived repository permissions consistently with existing repository access settings.
- Notification rules: enabled team notifications create or update notification subscriptions for team mentions; disabled notifications preserve mention indexing but suppress member fanout unless direct mention/participation/review request rules already subscribe the user.
- `web/src/app/orgs/[org]/teams/[teamSlug]/page.tsx`: replace the placeholder with a real detail surface using Editorial tabs/cards for Overview, Members, Repositories, and Child teams, with concrete links to people/repository settings where supported.
- Tests: extend Rust contracts for detail privacy, visible mentionability, secret-team access, parent-child permission cascade, repository review request lookup, notification fanout enabled/disabled, inherited repository rows, and no private member leakage; extend Vitest/Playwright for detail tabs, repository/member links, inherited chips, empty child teams, no dead controls, and screenshot `ralph/screenshots/build/org-admin-004-phase4-team-detail.jpg`.

**Verification**: focused Rust contracts for teams plus existing repository access and notification fanout tests, focused Vitest, focused Playwright detail smoke, then `make check && make test`; run full `make test-e2e` unless the wrapper instability is documented and direct Playwright passes.

---

## Phase 5: Docs, Browser QA Handoff, and Build Pass - finish org-admin-004

**Done**: [ ]

**Scope**: Document organization teams endpoints and complete final build-loop bookkeeping. Mark `org-admin-004.build_pass=true` only after list, empty state, create, nesting constraints, team detail, permissions, mentions, notifications, browser smoke, and QA handoff are verified.

**Key changes**:
- `web/src/lib/api-docs.ts`: document `GET /api/orgs/{org}/teams`, `POST /api/orgs/{org}/teams`, `GET /api/orgs/{org}/teams/{team_slug}`, and any narrow team notification/permission endpoints added in Phase 4, including auth, visibility, parent rules, secret nesting constraints, notification semantics, permission cascade, audit behavior, and standard error envelopes.
- `web/tests/api-docs.test.tsx`: assert organization teams endpoint paths, request/response shapes, visibility/privacy notes, secret nesting validation, team mention behavior, repository-permission cascade notes, notification notes, and no raw stack traces/secrets.
- `web/tests/e2e/organization-teams.spec.ts`: final smoke for empty directory, populated list, search/filter, new-team validation, visible team create, secret nested-team block, detail tabs, row navigation, repository/member links, docs link, mobile no-overflow, and no dead controls.
- `ralph/screenshots/build/`: save final empty state, populated directory, create form, validation error, detail overview, repository permissions, and mobile screenshots when test data allows.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `org-admin-004.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session DASHBOARD_E2E_SKIP_MIGRATIONS=1 make test-e2e`; browser smoke proves every visible organization teams tab, filter, row, CTA, detail link, form, radio group, selector, dialog/error state, docs link, and redirect has concrete behavior; mandatory Editorial banned-value scan returns zero matches.
