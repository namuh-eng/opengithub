# Structure Outline: Workflow Run Detail, Jobs, Logs, and Artifacts

**Ticket**: `actions-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/og-screens-4.jsx`, `target-docs/content/actions/how-tos/monitor-workflows/use-workflow-run-logs.md`, `target-docs/content/actions/how-tos/manage-workflow-runs/re-run-workflows-and-jobs.md`, `target-docs/content/actions/how-tos/manage-workflow-runs/cancel-a-workflow-run.md`, current `.qrspi/actions-001/structure.md`, current `.qrspi/actions-002/structure.md`, current `crates/api/src/domain/actions.rs`, current `crates/api/src/routes/actions.rs`, current Actions migrations, and current placeholder `web/src/app/[owner]/[repo]/actions/runs/[runId]/page.tsx`.
**Date**: 2026-05-02

## Phase 1: Run Detail Read Contract - one run exposes jobs, attempts, annotations, and artifacts

**Done**: [x]

**Scope**: Build the permission-aware backend contract for `/{owner}/{repo}/actions/runs/{run_id}` before replacing the placeholder UI. A repository reader can fetch the selected run, workflow breadcrumb, attempt list, job/sidebar summary, steps, annotations, artifacts, log availability, metadata, and action eligibility without leaking private repository data.

**Key changes**:
- `crates/api/migrations/*_actions_run_detail.*.sql`: add only missing additive tables/columns for run attempts, annotations, artifacts, log metadata/deletion state, rerun lineage, cancel/delete audit markers, and optional storage keys; preserve existing `actions_workflows`, `workflow_runs`, `workflow_jobs`, and `workflow_steps` contracts.
- `crates/api/src/domain/actions.rs`: add `ActionsRunDetail`, `ActionsRunDetailRun`, `ActionsRunAttempt`, `ActionsRunJobDetail`, `ActionsRunStepDetail`, `ActionsRunAnnotation`, `ActionsRunArtifact`, `ActionsRunActionState`, and helpers that join workflows, runs, jobs, steps, users, commits, pull requests, artifacts, and repository visibility.
- `crates/api/src/routes/actions.rs`: add or extend `GET /api/repos/{owner}/{repo}/actions/runs/{run_id}/detail`; keep the existing REST-ish `GET /actions/runs/{run_id}` stable or alias it to the richer detail contract when safe.
- `crates/api/tests/api_repository_actions_run_detail_contract.rs`: seed public/private repositories, runs with multiple attempts, queued/in-progress/completed/cancelled states, jobs/steps, annotations, artifacts, actors, commits, PR refs, and deleted-log markers; assert permission filtering, 404/403 envelopes, attempt ordering, job grouping, action eligibility, artifact metadata, and private redaction.

**Verification**: focused Rust run-detail contract tests, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, and same-env `make test`. Browser smoke is optional in this API-first phase.

---

## Phase 2: Editorial Run Detail Page - placeholder becomes the real run workspace

**Done**: [x]

**Scope**: Replace the placeholder `/{owner}/{repo}/actions/runs/{run_id}` page with an Editorial run detail workspace. The page shows workflow breadcrumb, large status/conclusion, run title/number, attempt selector, trigger metadata, summary cards, annotations, artifacts, and a left job sidebar that routes/focuses to job logs.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed run-detail fetch helpers, artifact/download URL builders, action-state DTOs, and cookie forwarding.
- `web/src/app/[owner]/[repo]/actions/runs/[runId]/page.tsx`: fetch the Phase 1 contract, preserve repository shell/tab activation, handle not-found/forbidden states, and render the run detail component.
- `web/src/components/RepositoryActionsRunPage.tsx`: new component using `.btn`, `.chip`, `.card`, `.input`, `.list-row`, `.tabs`, `.t-*`, and `var(--*)`; implement the two-column run layout from `og-screens-4.jsx`, responsive job sidebar, summary cards, annotation list, artifact table, run metadata, and selected job focus state.
- `web/tests/repository-actions-run.test.tsx`: cover header metadata, attempt selector links, job sidebar focus, summary cards, annotations, artifact table, deleted-log unavailable state, accessible controls, no `href="#"`, and Editorial-token-only styling.
- `web/tests/e2e/repository-actions-run.spec.ts`: signed-session browser smoke for a seeded run detail page with jobs, annotations, artifacts, and screenshot `ralph/screenshots/build/actions-003-phase2-run-detail.jpg`.

**Verification**: focused run-page Vitest, focused run-page Playwright smoke, `make check`, `make test`, and focused same-env `make test-e2e`.

---

## Phase 3: Job Logs and Artifact Downloads - log and artifact actions are data-backed

**Done**: [ ]

**Scope**: Make the selected job log and artifact controls real. Job links focus the log pane, log search filters lines without losing context, line anchors/copy links are stable, download-log actions return a backend URL or text stream, and artifact download/copy affordances use short-lived signed URLs or local dev equivalents.

**Key changes**:
- `crates/api/src/domain/actions.rs`: add log line storage/read helpers, bounded log pagination/search, deleted-log checks, artifact download token generation, and artifact digest/size formatting; use S3-compatible storage keys when configured and deterministic local responses in tests.
- `crates/api/src/routes/actions.rs`: add `GET /api/repos/{owner}/{repo}/actions/jobs/{job_id}/logs`, `GET /api/repos/{owner}/{repo}/actions/jobs/{job_id}/logs/download`, and `GET /api/repos/{owner}/{repo}/actions/artifacts/{artifact_id}/download` with read permission and no private metadata leakage.
- `web/src/app/[owner]/[repo]/actions/{jobs,artifacts}/.../route.ts`: same-origin proxies for log and artifact endpoints so browser controls remain cookie-backed.
- `web/src/components/RepositoryActionsRunPage.tsx`: render the selected job log pane with search, follow/pause affordance for live runs, line anchors/copy buttons, deleted-log empty state, download-log controls, artifact copy/download actions, and clear success/error feedback.
- Rust, Vitest, and Playwright tests: cover log read/search bounds, deleted logs hiding downloads, artifact signed URL shape, line-anchor navigation, copy/download feedback, and browser smoke screenshot `ralph/screenshots/build/actions-003-phase3-logs-artifacts.jpg`.

**Verification**: focused Rust log/artifact contract tests, focused run-page Vitest, focused run-page Playwright smoke, `make check`, `make test`, and focused same-env `make test-e2e`.

---

## Phase 4: Rerun, Cancel, Delete Logs, and API Docs - run actions mutate real state

**Done**: [ ]

**Scope**: Wire every run action to a real backend mutation. Writers can rerun all jobs, rerun failed jobs, rerun a specific job, cancel queued/in-progress runs, and delete logs after confirmation; the UI updates from returned state and `/docs/api` documents the run detail and mutation endpoints.

**Key changes**:
- `crates/api/src/domain/actions.rs`: add `rerun_workflow_run`, `cancel_workflow_run`, and `delete_workflow_run_logs` with write-permission checks, rerun attempt limits, failed-job filtering, specific-job validation, queued job/lease creation, status transitions, log deletion markers, audit events, and idempotent conflict/error envelopes.
- `crates/api/src/routes/actions.rs`: add `POST /api/repos/{owner}/{repo}/actions/runs/{run_id}/rerun`, `POST /api/repos/{owner}/{repo}/actions/runs/{run_id}/cancel`, and `DELETE /api/repos/{owner}/{repo}/actions/runs/{run_id}/logs`; support body variants for all/failed/job-specific reruns.
- `web/src/app/[owner]/[repo]/actions/runs/[runId]/{rerun,cancel,logs}/route.ts`: same-origin mutation proxies that forward cookies and preserve Rust error envelopes.
- `web/src/components/RepositoryActionsRunPage.tsx`: add Editorial confirmation flows, rerun dropdown, job-specific rerun buttons, cancel visibility only for queued/in-progress runs, delete-log confirmation, optimistic pending states, and returned-state refresh without dead controls.
- `web/src/lib/api-docs.ts` and `/docs/api`: document run detail, job logs, artifact download, rerun, cancel, and delete-log endpoints with permission notes, response shapes, and standard errors.
- Tests: cover unauthorized/forbidden writes, rerun limit, failed-only and job-specific reruns, cancel state constraints, delete-log idempotency, audit rows, docs anchors, and browser mutation smoke `ralph/screenshots/build/actions-003-phase4-run-actions.jpg`.

**Verification**: focused Rust mutation contract tests, focused run-page/docs Vitest, focused run-page Playwright smoke, `make check`, `make test`, and same-env `make test-e2e`.

---

## Phase 5: Final Run Detail Guardrails and QA Handoff - complete actions-003 safely

**Done**: [ ]

**Scope**: Harden the full run detail surface before setting `actions-003.build_pass=true`. Validate permissions, run/action state matrices, job/sidebar navigation, log/artifact behavior, mobile layout, visual compliance, browser evidence, QA hints, and bookkeeping.

**Key changes**:
- `crates/api/tests/api_repository_actions_run_detail_contract.rs`: final matrix for anonymous public reads, private denial, owner private access, missing runs, run statuses/conclusions, attempt selection, annotations, artifacts, deleted logs, rerun/cancel/delete-log side effects, audit records, and queue leases.
- `web/tests/repository-actions-run.test.tsx` and `web/tests/e2e/repository-actions-run.spec.ts`: final dead-control sweep, keyboard/focus behavior, job/sidebar routing, mutation dialogs, artifact/download affordances, no horizontal overflow, and screenshots `ralph/screenshots/build/actions-003-phase5-final-desktop.jpg` and `ralph/screenshots/build/actions-003-phase5-final-mobile.jpg`.
- Mandatory Editorial banned-value scan before commit: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`: append honest notes for real runner execution not being part of this feature, S3 signed-URL/dev-storage limits, rerun queue worker behavior, log streaming/polling race cases, artifact retention edge cases, and seeded check-suite/check-run coverage.
- `prd.json`: set only `actions-003.build_pass` to `true` after every phase passes; leave `qa_pass=false`.
- `build-progress.txt`: append final summary, verification evidence, changed files, and known risks.

**Verification**: focused run-detail Rust contract tests, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`, and mandatory Editorial banned-value scan using the local compatible `rg -n -e` form.
