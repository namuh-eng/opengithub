# Structure Outline: Workflow-Specific Actions Runs and Manual Dispatch

**Ticket**: `actions-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/og-screens-4.jsx`, `target-docs/content/actions/how-tos/manage-workflow-runs/manually-run-a-workflow.md`, current `.qrspi/actions-001/structure.md`, current `crates/api/src/domain/actions.rs`, current `crates/api/src/routes/actions.rs`, current Actions migrations, and current `web/src/components/RepositoryActionsPage.tsx`.
**Date**: 2026-05-02

## Phase 1: Workflow Detail Read Contract - one workflow can own its page

**Done**: [x]

**Scope**: Build the permission-aware backend contract for `/{owner}/{repo}/actions/workflows/{workflow_file}` before changing the UI. A repository reader can fetch the selected workflow, the shared Actions rail, workflow-scoped run rows, source-file metadata, default-branch dispatch capability, repository refs, filter options without the Workflow filter, and invalid-workflow state.

**Key changes**:
- `crates/api/migrations/*_workflow_dispatch.*.sql`: add only missing additive columns/tables for workflow source metadata and parsed dispatch state, such as `actions_workflows.source_blob_id`, `source_sha`, `source_branch`, `yaml_parse_error`, `dispatch_inputs jsonb`, and `dispatch_enabled`; keep existing workflow/run contracts intact.
- `crates/api/src/domain/actions.rs`: add `ActionsWorkflowDetail`, `ActionsWorkflowDetailWorkflow`, `WorkflowDispatchInput`, `WorkflowDispatchSpec`, and `ActionsWorkflowDetailQuery`; reuse the actions-001 run row DTOs while forcing the workflow filter to the selected workflow.
- `crates/api/src/routes/actions.rs`: add `GET /api/repos/{owner}/{repo}/actions/workflows/{workflow_file}/dashboard` or equivalent suffix-safe route that accepts encoded workflow file paths plus `q`, `event`, `status`, `branch`, `actor`, `page`, and `pageSize`; reject unknown workflow paths with the standard 404 envelope.
- `crates/api/tests/api_repository_actions_workflow_detail_contract.rs`: seed public/private repositories, active/disabled workflows, workflow-specific runs, repository refs, parsed dispatch inputs, and invalid YAML; assert permission filtering, run scoping, removed Workflow filter, source link payload, dispatch visibility, invalid callout payload, and pagination bounds.

**Verification**: focused Rust workflow-detail contract tests, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, and same-env `make test`. Browser smoke is optional in this API-first phase.

---

## Phase 2: Editorial Workflow Page - selected workflow run history replaces the generic list

**Done**: [x]

**Scope**: Build `/{owner}/{repo}/actions/workflows/{workflow_file}` as an Editorial workflow-specific page. The left Actions rail stays visible with the selected workflow highlighted; the main pane shows workflow name, source workflow file link, workflow options, filter search, Event/Status/Branch/Actor filters, and scoped run rows.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed workflow-detail fetch helpers and URL builders that encode workflow file paths safely and forward cookies.
- `web/src/app/[owner]/[repo]/actions/workflows/[...workflowFile]/page.tsx`: fetch the Phase 1 contract, preserve repository shell/tab activation, and render not-found/error states without leaking private repository metadata.
- `web/src/components/RepositoryActionsWorkflowPage.tsx`: new component using `.btn`, `.chip`, `.card`, `.input`, `.list-row`, `.tabs`, `.t-*`, and `var(--*)`; share small run-row/status helpers with `RepositoryActionsPage` when it reduces duplication.
- `web/tests/repository-actions-workflow.test.tsx`: cover selected rail state, source YAML link, absence of Workflow filter, scoped run navigation, invalid workflow callout, accessible controls, no `href="#"`, and Editorial-token-only styling.
- `web/tests/e2e/repository-actions-workflow.spec.ts`: signed-session browser smoke for a seeded workflow page with scoped runs and save `ralph/screenshots/build/actions-002-phase2-workflow-page.jpg`.

**Verification**: focused workflow-page Vitest, focused workflow-page Playwright smoke, `make check`, `make test`, and focused same-env `make test-e2e`.

---

## Phase 3: Manual Dispatch Contract and Dialog - workflow_dispatch creates a queued run

**Done**: [ ]

**Scope**: Make the Run workflow affordance real for workflows whose default-branch YAML defines `workflow_dispatch`. A writer can open the dialog, choose a branch/ref, fill dynamic inputs, submit, and see the newly queued run inserted at the top.

**Key changes**:
- `crates/api/src/domain/actions.rs`: add `DispatchWorkflowRun`, `DispatchWorkflowRunInput`, `dispatch_workflow_run`, YAML dispatch input validation, ref resolution against `repository_git_refs`, required/default/type handling for text/choice/boolean inputs, and seed records for `workflow_runs`, `workflow_run_attempts`, `workflow_jobs`, `check_suites`, `check_runs`, and the existing job queue/lease table if present.
- `crates/api/src/routes/actions.rs`: add `POST /api/repos/{owner}/{repo}/actions/workflows/{workflow_file}/dispatches` with write permission, default-branch dispatch checks, ref validation, bounded input count/size, and standard 201/error envelopes.
- `web/src/app/[owner]/[repo]/actions/workflows/[...workflowFile]/dispatches/route.ts`: same-origin proxy that forwards cookies and returns API envelopes unchanged.
- `web/src/components/RepositoryActionsWorkflowPage.tsx`: add a Run workflow button only when dispatch is enabled, an Editorial modal/dialog with branch select plus dynamic compact label/input/select/checkbox controls, optimistic pending state, validation feedback, and queued-run insertion after success.
- Rust, Vitest, and Playwright tests: cover missing write permission, disabled dispatch, invalid ref, missing required input, choice validation, boolean serialization, successful queue side effects, dialog payloads, and browser success flow; save `ralph/screenshots/build/actions-002-phase3-run-workflow.jpg`.

**Verification**: focused Rust dispatch contract tests, focused workflow-page Vitest, focused workflow dispatch Playwright smoke, `make check`, `make test`, and focused same-env `make test-e2e`.

---

## Phase 4: Invalid YAML, Workflow Options, and API Docs - no dead workflow controls remain

**Done**: [ ]

**Scope**: Finish workflow-specific navigation and developer-facing documentation. Invalid YAML disables dispatch with a clear callout; workflow options and source links resolve to concrete routes; `/docs/api` documents workflow detail and dispatch.

**Key changes**:
- `crates/api/src/domain/actions.rs`: surface stable parse-error metadata and last parse timestamp without exposing raw stack traces; keep invalid workflows visible in the rail and detail page.
- `web/src/components/RepositoryActionsWorkflowPage.tsx`: wire workflow options to concrete settings/source/docs destinations, make source file link open `/{owner}/{repo}/blob/{branch}/{workflow_path}`, add invalid workflow callout, and ensure every empty state has a working recovery action.
- `web/src/app/[owner]/[repo]/actions/workflows/[...workflowFile]/settings/page.tsx` or a chosen concrete destination: add a thin repository workspace page if no better settings route exists yet.
- `web/src/lib/api-docs.ts` and `/docs/api`: document workflow detail read, workflow-scoped run list, manual dispatch request/response, validation errors, permissions, branch/ref behavior, and invalid YAML state.
- `web/tests/api-docs.test.tsx`, `web/tests/repository-actions-workflow.test.tsx`, and navigation route tests: assert docs anchors, source/settings links, invalid-YAML behavior, disabled dispatch, and no dead controls.

**Verification**: focused docs/navigation Vitest, focused workflow-page Playwright smoke, `make check`, `make test`, and same-env `make test-e2e`.

---

## Phase 5: Final Workflow Guardrails and QA Handoff - complete actions-002 safely

**Done**: [ ]

**Scope**: Harden the full workflow-specific Actions surface before setting `actions-002.build_pass=true`. Validate permissions, run scoping, dispatch side effects, invalid YAML, mobile layout, visual compliance, browser evidence, QA hints, and bookkeeping.

**Key changes**:
- `crates/api/tests/api_repository_actions_workflow_detail_contract.rs`: final matrix for public/private access, anonymous public reads, disabled workflows, invalid YAML, default-branch-only dispatch, write-permission dispatch, input validation, queued run ordering, seeded check records, and no cross-workflow run leakage.
- `web/tests/repository-actions-workflow.test.tsx` and `web/tests/e2e/repository-actions-workflow.spec.ts`: final dead-control sweep, selected rail state, filter combinations, dialog keyboard/focus behavior, no horizontal overflow, and screenshots `ralph/screenshots/build/actions-002-phase5-final-desktop.jpg` and `ralph/screenshots/build/actions-002-phase5-final-mobile.jpg`.
- Mandatory Editorial banned-value scan before commit: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`: append honest notes for real runner execution not being part of this feature, YAML parser edge cases, branch/ref race conditions, dispatch queue worker limits, and seeded check-suite/check-run coverage.
- `prd.json`: set only `actions-002.build_pass` to `true` after every phase passes; leave `qa_pass=false`.
- `build-progress.txt`: append final summary, verification evidence, changed files, and known risks.

**Verification**: focused workflow-detail/dispatch Rust contract tests, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`, and mandatory Editorial banned-value scan using the local compatible `rg -n -e` form.
