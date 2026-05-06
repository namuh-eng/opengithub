# Structure Outline: projects-007 Project Settings, Permissions, Templates, and Danger Zone

**Ticket**: `projects-007`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, existing Projects outlines in `.qrspi/projects-001/structure.md` through `.qrspi/projects-006/structure.md`, organization policy outline `.qrspi/org-admin-005/structure.md`, `crates/api/migrations/202605060006_projects_v2_foundation.up.sql`, `crates/api/src/domain/projects.rs`, `crates/api/src/routes/projects.rs`, `web/src/components/ProjectFieldSettingsPage.tsx`, `web/src/components/ProjectWorkflowSettingsPage.tsx`, `web/src/components/ProjectWorkspacePage.tsx`, and `web/src/lib/api-docs.ts`.
**Date**: 2026-05-06

## Phase 1: Settings Read Contract - general settings, policy, access, repositories, templates, and danger state are inspectable

**Done**: [x]

**Scope**: Add the authenticated Rust read contract for project settings. The response covers project metadata, README/description, visibility and organization policy constraints, default and linked repositories, status update history, access grants, teams/users eligible for grants, template state, closed/deleted capabilities, viewer role, and audit-safe capability flags without mutation UI yet.

**Key changes**:
- `crates/api/migrations/`: add only missing additive project settings metadata, such as project README revisions, deleted/closed timestamps if absent, status update actor indexes, repository-link metadata, team grant rows if existing `project_permissions` cannot represent teams, and project settings audit indexes.
- `crates/api/src/domain/projects.rs`: add `ProjectSettings`, `ProjectSettingsGeneral`, `ProjectSettingsPolicy`, `ProjectSettingsRepositoryLink`, `ProjectSettingsAccessGrant`, `ProjectSettingsTeamGrant`, `ProjectSettingsStatusUpdate`, `ProjectSettingsTemplate`, `ProjectSettingsDangerState`, and capability DTOs.
- `crates/api/src/routes/projects.rs`: expose `GET /api/projects/{project_id}/settings` plus owner/number lookup helpers consistent with field/workflow settings, enforcing project privacy, organization policy, repository visibility filtering, and standard no-secret error envelopes.
- `web/src/lib/api.ts`: add typed settings DTOs and signed-cookie helpers for user/org project-number routes.
- `crates/api/tests/projects_settings_contract.rs`: cover owner/admin/member/read-only/outsider reads, private project privacy, organization policy locks, linked repository filtering, access grant shape, template state, closed/deleted capabilities, and no-secret errors.

**Verification**: focused Rust contract tests, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, web TypeScript for DTOs, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial General Settings UI - metadata, README, visibility, default repository, and status panels render with real forms

**Done**: [x]

**Scope**: Build `/orgs/{org}/projects/{number}/settings` and user-project equivalents on the Editorial settings shell using the Phase 1 read contract. The page shows General, Access, Fields, Workflows, Templates, and Danger Zone navigation, with concrete forms for title, short description, README Markdown, visibility, default repository, and status update drafting.

**Key changes**:
- `web/src/app/[owner]/projects/[number]/settings/page.tsx` and `web/src/app/orgs/[org]/projects/[number]/settings/page.tsx`: load settings through signed-cookie helpers and render forbidden/not-found/closed states consistently with existing Projects settings pages.
- `web/src/components/ProjectSettingsPage.tsx`: add the Editorial settings surface with breadcrumb/header, left settings nav, metadata form, README editor, visibility selector, default repository selector, latest status card, status update panel, policy lock copy, and permission-disabled controls.
- `web/src/lib/navigation.ts`: add stable href builders for General, Access, Fields, Workflows, Templates, Danger Zone, workspace return, repository links, and status anchors.
- Controls must submit to real same-origin routes in later phases, open route-backed panels/dialogs, or be disabled with an explanation. No `href="#"`, inert handlers, GitHub colors, Primer imports, or Octicons.
- `web/tests/project-settings-page.test.tsx`: cover user/org rendering, settings nav links, metadata field defaults, policy-disabled visibility, repository selector options, status panel controls, read-only disabled states, no dead controls, mobile wrapping, and Editorial token guardrails.

**Verification**: focused Vitest, web TypeScript, focused Biome, mandatory Editorial banned-value scan, Playwright smoke when seeded data is available saving `ralph/screenshots/build/projects-007-phase2-general-settings.jpg`, then `make check && make test`.

---

## Phase 3: General Settings Mutations - metadata, README, visibility, repository links, templates, and status updates persist

**Done**: [x]

**Scope**: Make the General and Templates controls real. Admins can rename projects, edit description/README, change visibility when policy allows it, choose a default repository, add/remove repository links, publish status updates, and toggle template behavior with audit evidence.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement settings update helpers with project admin/write checks, organization policy enforcement for visibility/projects, stale `expectedUpdatedAt` conflicts, README revision writes, default repository write-permission checks, linked repository add/remove validation, status update validation, template flag/copy-source persistence, and audit events.
- `crates/api/src/routes/projects.rs`: expose focused mutations such as `PATCH /api/projects/{project_id}/settings`, `PUT/DELETE /api/projects/{project_id}/repositories/{repository_id}`, `POST /api/projects/{project_id}/status-updates`, and `PATCH /api/projects/{project_id}/template`.
- `web/src/app/api/projects/[projectId]/settings/route.ts`, `repositories/[repositoryId]/route.ts`, `status-updates/route.ts`, and `template/route.ts`: forward signed cookies to Rust and preserve standard JSON envelopes.
- `ProjectSettingsPage.tsx`: wire Save buttons, repository link controls, status publish form, template toggle, pending/error/success states, conflict copy, and refreshed settings state.
- Tests cover rename/README/status/template persistence, invalid visibility denial, organization policy lock denial, repository permission denial, duplicate repository link rejection, default repository behavior for new issues, audit rows, UI payloads, and no client-only fake updates.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-007-phase3-general-mutations.jpg` when possible, then `make check && make test`; run `make test-e2e` when local DB/dev servers are healthy.

---

## Phase 4: Access Settings - users and teams receive project roles with policy boundaries

**Done**: [x]

**Scope**: Add the Access settings page and real grant mutations. Admins can list collaborators and teams, add users/teams, change roles, and remove grants while organization base permission and owner/admin protections remain intact.

**Key changes**:
- `crates/api/src/domain/projects.rs`: add access grant helpers for direct users and teams, role validation (`read`, `write`, `admin`), organization membership checks, team ownership checks, base permission inheritance, last-admin protection, stale conflicts, permission recalculation, and audit events.
- `crates/api/src/routes/projects.rs`: expose `GET /api/projects/{project_id}/settings/access`, `POST /api/projects/{project_id}/access-grants`, `PATCH /api/projects/{project_id}/access-grants/{grant_id}`, and `DELETE /api/projects/{project_id}/access-grants/{grant_id}`.
- `web/src/app/[owner]/projects/[number]/settings/access/page.tsx` and org equivalent: render the Access settings page using the shared settings shell.
- `web/src/components/ProjectAccessSettingsPage.tsx`: add collaborator/team tables, role menus, add-grant dialog, inherited permission chips, remove confirmation, read-only disabled states, and concrete links to org/team/repository surfaces.
- Tests cover user/team add, role change, remove, non-member denial, team outside org denial, last-admin protection, read-only denial, audit rows, UI dialog payloads, and mobile table overflow.

**Verification**: focused Rust access contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-007-phase4-access-settings.jpg` when possible, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 5: Danger Zone Lifecycle - close, reopen, and delete are confirmed, audited, and reflected everywhere

**Done**: [ ]

**Scope**: Add the Danger Zone page and lifecycle mutations. Admins can close/reopen projects and delete projects only after typed confirmation; closed projects remain readable but mutation-disabled where appropriate, and deleted projects disappear from normal project lists without leaking private metadata.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement close, reopen, and soft-delete helpers with admin checks, typed confirmation, last-project/list behavior, project item/workflow/archive side-effect boundaries, audit events, and privacy-preserving deleted/not-found responses.
- `crates/api/src/routes/projects.rs`: expose `POST /api/projects/{project_id}/close`, `POST /api/projects/{project_id}/reopen`, and `DELETE /api/projects/{project_id}` or equivalent lifecycle routes.
- `web/src/app/[owner]/projects/[number]/settings/danger/page.tsx` and org equivalent: render lifecycle state from the settings read contract.
- `web/src/components/ProjectDangerZonePage.tsx`: add close/reopen/delete cards, typed confirmation dialogs, disabled policy/permission states, pending/error/success feedback, and post-delete navigation to the owner/org Projects list.
- Update workspace/settings read paths to respect closed/deleted state consistently and disable unrelated mutation controls for closed projects.
- Tests cover close/reopen/delete confirmations, wrong confirmation rejection, read-only denial, deleted privacy, closed mutation disablement, audit rows, UI dialogs, and navigation after delete.

**Verification**: focused Rust lifecycle contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-007-phase5-danger-zone.jpg` when possible, then `make check && make test`; run `make test-e2e` when local DB/dev servers are healthy.

---

## Phase 6: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `projects-007` only after settings reads, Editorial settings UI, general/template/status/repository mutations, access grants, danger lifecycle, docs, screenshots, and QA handoff are verified. Do not implement project insights, exports, billing, full organization policy management, repository issue creation beyond default-repository routing, custom field/workflow behavior already covered by `projects-004` and `projects-006`, or GitHub visual styling as part of this feature.

**Key changes**:
- `web/src/lib/api-docs.ts`: document project settings reads, metadata/README/visibility/default-repository updates, repository link management, status updates, template toggles, access grant endpoints, close/reopen/delete lifecycle, auth/privacy, organization policy locks, permissions, stale conflicts, audit/log side effects, and standard errors.
- `web/tests/e2e/projects-settings.spec.ts`: final signed-in browser smoke for org and user project settings covering General, Access, Templates, Danger Zone, all menus/forms/dialogs, success/error feedback, no dead controls, desktop/mobile screenshots, and bounded overflow.
- `ralph/screenshots/build/`: save final evidence screenshots for general settings, status update, repository selector, access grants, template toggle, danger confirmation, closed state, and mobile.
- `qa-hints.json`: append QA targets for concurrent settings edits, long README/status Markdown, private repository links, organization policy visibility locks, default repository issue routing, large access lists, team membership drift, template copy behavior, close/reopen races, delete privacy, audit rows, and mobile settings layout.
- `build-progress.txt`, `.qrspi/projects-007/structure.md`, and `prd.json`: record evidence and set `projects-007.build_pass=true` only after all phases pass; leave `qa_pass=false`.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.

**Verification**: focused Rust/Vitest/Playwright checks, `make check`, `make test`, `make test-e2e` when local DB/dev servers are healthy, browser screenshots under `ralph/screenshots/build/`, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan with zero matches.
