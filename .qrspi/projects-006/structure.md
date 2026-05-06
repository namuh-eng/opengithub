# Structure Outline: projects-006 Built-in Project Workflows and Automation

**Ticket**: `projects-006`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, existing Projects outlines in `.qrspi/projects-001/structure.md` through `.qrspi/projects-005/structure.md`, Actions outlines in `.qrspi/actions-001/structure.md` through `.qrspi/actions-003/structure.md`, `crates/api/migrations/202605060006_projects_v2_foundation.up.sql`, `crates/api/src/domain/projects.rs`, `crates/api/src/routes/projects.rs`, `crates/api/src/domain/actions.rs`, `crates/api/src/routes/actions.rs`, `web/src/components/ProjectFieldSettingsPage.tsx`, `web/src/components/ProjectWorkspacePage.tsx`, `web/src/lib/api-docs.ts`, and Projects workflow docs under `target-docs/content/issues/planning-and-tracking-with-projects/`.
**Date**: 2026-05-06

## Phase 1: Workflow Settings Read Contract - default automation is inspectable

**Done**: [x]

**Scope**: Add the authenticated read contract for `/orgs/{org}/projects/{number}/workflows` and user-project equivalents. The response returns project metadata, default workflow definitions, current enablement, editable rule configuration, eligible status fields/options, repository choices for auto-add, recent execution logs, and viewer capabilities without adding mutation UI yet.

**Key changes**:
- `crates/api/migrations/`: add only missing additive workflow metadata around the existing `project_workflows` table, such as stable workflow keys, rule rows if needed, actor/source attribution, execution log rows, last-run status, repository allow-lists, and indexes by project/workflow/event/created time.
- `crates/api/src/domain/projects.rs`: add `ProjectWorkflowSettings`, `ProjectWorkflowDefinition`, `ProjectWorkflowRule`, `ProjectWorkflowEligibleField`, `ProjectWorkflowRepositoryTarget`, `ProjectWorkflowExecutionLog`, and permission/capability DTOs. Seed default workflow rows for closed issue/PR to Done and merged PR to Done when a project first needs workflow settings.
- `crates/api/src/routes/projects.rs`: expose `GET /api/projects/{project_id}/workflows` plus owner/number convenience lookup if consistent with field settings, enforcing project privacy, repository visibility filtering, and no-secret error envelopes.
- `web/src/lib/api.ts`: add typed workflow-settings DTOs and signed-cookie helpers without JS-side auth.
- `crates/api/tests/projects_workflows_contract.rs`: cover default initialization, enabled default workflows, missing Done/status option fallback, private repository target filtering, read-only capability flags, execution log shape, and no-secret errors.

**Verification**: focused Rust contract tests, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, web TypeScript for DTOs, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Workflows Settings Page - automation cards render without dead controls

**Done**: [x]

**Scope**: Render the Projects settings Workflows page using the Phase 1 read contract. The page shows the settings shell, default workflow cards, enabled/disabled toggles, event descriptions, recent activity, automation attribution as `@github-project-automation` equivalent, and permission-aware Edit/Turn on controls.

**Key changes**:
- `web/src/app/[owner]/projects/[number]/workflows/page.tsx`, `web/src/app/orgs/[org]/projects/[number]/workflows/page.tsx`, and settings aliases if needed: load workflow settings through signed-cookie helpers and render honest forbidden/not-found states.
- `web/src/components/ProjectWorkflowSettingsPage.tsx`: add the Editorial settings surface with project breadcrumb/header, settings sidebar links for General/Access/Fields/Workflows/Templates/Danger Zone, workflow card list, enabled chips, recent log rows, empty/disabled workflow copy, and stable responsive layout.
- `web/src/lib/navigation.ts`: add stable href builders for workflow settings, individual workflow edit state, fields settings, workspace return, and archived item return paths.
- `ProjectFieldSettingsPage.tsx` or shared settings-shell helper: replace the disabled Workflows sidebar item with a concrete link while preserving existing Fields behavior.
- Controls must navigate, open a real edit panel/dialog, submit to a real route in later phases, or be permission-disabled with explanatory copy. No `href="#"`, inert handlers, GitHub visual colors, Primer imports, or Octicons.
- `web/tests/project-workflow-settings-page.test.tsx`: cover owner/org rendering, default workflow cards, enabled/disabled states, edit panel opening, settings sidebar links, permission-disabled controls, no dead links, mobile text fit, unsafe-markup guardrails, and Editorial token usage.

**Verification**: focused Vitest, web TypeScript, focused Biome, mandatory Editorial banned-value scan, Playwright smoke when seeded data is available saving `ralph/screenshots/build/projects-006-phase2-workflows-settings.jpg`, then `make check && make test`.

---

## Phase 3: Workflow Configuration Mutations - toggles and rule editors persist

**Done**: [x]

**Scope**: Make workflow enable/disable, condition/filter edits, target status value selection, repository auto-add selectors, archive criteria, and close-on-status behavior real. Authorized users can save and turn on workflows; read-only viewers see disabled controls.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement workflow update helpers with project write/admin checks, stable workflow-key validation, supported event validation, target status field/option validation, repository target permission checks, filter/condition bounds, auto-archive criteria validation, stale `expectedUpdatedAt` conflicts, workflow execution-log writes, and audit events.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/workflows/{workflow_id}`, `POST /api/projects/{project_id}/workflows/{workflow_id}/enable`, and `POST /api/projects/{project_id}/workflows/{workflow_id}/disable` or a single patch route if that keeps the contract simpler.
- `web/src/app/api/projects/[projectId]/workflows/[workflowId]/route.ts` and enable/disable proxy routes: forward signed cookies to Rust.
- `ProjectWorkflowSettingsPage.tsx`: wire toggle buttons, Edit dialog, condition/filter builder, target status selectors, repository selector, archive criteria form, Save, Save and turn on, pending/error/success states, conflict copy, and refreshed workflow cards.
- Tests cover enable/disable, stale updates, invalid field/option/repository denial, read-only denial, rule persistence, execution-log/audit evidence, proxy cookie forwarding, UI success/error states, and no local-only fake updates.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-006-phase3-workflow-edit.jpg` when possible, then `make check && make test`; run `make test-e2e` when local DB/dev servers are healthy.

---

## Phase 4: Item-State Automation Engine - issue/PR events update project fields safely

**Done**: [x]

**Scope**: Implement the server-side automation engine for built-in project workflows. Repository issue/PR close, reopen, merge, and item-add events evaluate enabled project workflows, update project item field values, add project item events, and preserve repository/project permission boundaries.

**Key changes**:
- `crates/api/src/domain/projects.rs`: add workflow execution helpers for `item_added`, `issue_closed`, `issue_reopened`, `pull_request_closed`, `pull_request_merged`, and status-driven close behavior. Match rule filters, update `project_item_field_values`, write `project_item_events`, append `workflow_execution_logs`, and avoid duplicate updates with idempotency keys.
- Issue and pull request domain modules/routes: call the project automation engine from existing state-change flows without bypassing repository write checks or creating circular updates. Close-on-status must close linked issues/PRs only when the actor already has repository permission.
- Auto-archive: evaluate completed item age/status criteria, archive matching items, and preserve archived/restored actor metadata as automation attribution.
- Notifications/audit: write audit rows and notifications only for linked issue/PR state changes that already produce repository-visible side effects; draft/project-only changes stay project-local.
- `crates/api/tests/projects_workflows_execution_contract.rs`: cover closed issue to Done, merged PR to Done, reopened item reset when configured, item-added default status, close-on-status permission denial, auto-archive criteria, idempotent repeated events, audit/log evidence, and private linked resource boundaries.

**Verification**: focused Rust contract tests against seeded issues/PRs/projects, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke is optional for this backend-heavy phase.

---

## Phase 5: Actions and GraphQL Automation Hooks - external triggers use the same engine

**Done**: [x]

**Scope**: Allow automation to be invoked from Actions workflow runs and GraphQL-style mutations without introducing a full GraphQL API. This phase exposes a bounded REST-equivalent internal contract that Actions can call, records `actions_workflow_runs` attribution where available, and proves the same permission and idempotency rules as Phase 4.

**Key changes**:
- `crates/api/src/domain/projects.rs`: add a narrow project automation invocation helper that accepts an actor, optional `actions_workflow_run_id`, project id, item id/source selector, field updates, and workflow key/idempotency metadata.
- `crates/api/src/routes/projects.rs`: expose an internal/authenticated route such as `POST /api/projects/{project_id}/automation/invocations` for PAT/session authorized callers with project write access; document this as the GraphQL/Actions hook surface for MVP rather than shipping a separate GraphQL server.
- `crates/api/src/domain/actions.rs`: when manual dispatch or workflow-run APIs need project automation attribution, call the new helper rather than writing project fields directly.
- `workflow_execution_logs`: record source `ui`, `system`, `actions`, or `graphql`, the automation actor label, result status, skipped reason, and affected item/field ids.
- Tests cover session/PAT authorization, project permission denial, Actions run attribution, GraphQL-style mutation payload validation, idempotency, skipped private items, execution logs, audit rows, and no direct field writes outside the project workflow engine.

**Verification**: focused Rust contract tests across Projects and Actions, focused API hardening tests for auth/envelopes, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` only if a browser-visible flow was changed.

---

## Phase 6: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `projects-006` only after workflow settings reads, Editorial settings UI, workflow configuration mutations, built-in automation execution, Actions/GraphQL hook invocation, docs, screenshots, and QA handoff are verified. Do not implement project insights, exports, full GraphQL schema/query execution, custom workflow languages, scheduled background runners beyond auto-archive criteria, repository Actions YAML authoring, access settings, or GitHub visual styling as part of this feature.

**Key changes**:
- `web/src/lib/api-docs.ts`: document workflow settings reads, workflow update/enable/disable, automation invocations, built-in event vocabulary, condition/filter syntax, repository target selection, archive criteria, close-on-status behavior, Actions/GraphQL attribution, auth/privacy, permissions, idempotency, audit/log side effects, and standard errors.
- `web/tests/e2e/projects-workflows.spec.ts`: final signed-in browser smoke for org and user project workflows covering default cards, edit dialog, toggle on/off, condition/filter save, repository auto-add selector, archive criteria save, activity log visibility, no dead controls, desktop/mobile screenshots, and bounded overflow.
- `ralph/screenshots/build/`: save final evidence screenshots for workflows list, edit dialog, disabled/read-only state, repository selector, archive criteria, activity log, and mobile.
- `qa-hints.json`: append QA targets for concurrent workflow edits, stale toggles, missing Done/status option, private repository auto-add targets, close-on-status permission denial, duplicate event idempotency, auto-archive age boundaries, Actions run attribution, GraphQL-style invocation auth, audit rows, notifications, and mobile settings layout.
- `build-progress.txt`, `.qrspi/projects-006/structure.md`, and `prd.json`: record evidence and set `projects-006.build_pass=true` only after all phases pass; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, `make check`, `make test`, `make test-e2e` when local DB/dev servers are healthy, browser screenshots under `ralph/screenshots/build/`, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan with zero matches.
