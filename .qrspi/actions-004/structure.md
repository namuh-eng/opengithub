# Structure Outline: Actions Job Log Viewer

**Ticket**: `actions-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/og-screens-4.jsx`, `target-docs/content/actions/get-started/quickstart.md`, `target-docs/content/actions/how-tos/manage-workflow-runs/download-workflow-artifacts.md`, current `.qrspi/actions-002/structure.md`, current `.qrspi/actions-003/structure.md`, current `crates/api/src/domain/actions.rs`, current `crates/api/src/routes/actions.rs`, and current `web/src/components/RepositoryActionsRunPage.tsx`.
**Date**: 2026-05-02

## Phase 1: Job Log Read Contract - one job can own its viewer page

**Done**: [x]

**Scope**: Build or extend the permission-aware backend contract for `/{owner}/{repo}/actions/runs/{run_id}/jobs/{job_id}` before changing the dedicated UI. A repository reader can fetch the run breadcrumb, selected job, job siblings for the left sidebar, ordered steps, annotations, log chunks, search metadata, display preferences, and unavailable/deleted-log state without leaking private repository data.

**Key changes**:
- `crates/api/src/domain/actions.rs`: add `ActionsJobLogDetail`, `ActionsJobLogRunSummary`, `ActionsJobLogStep`, `ActionsJobLogChunk`, `ActionsJobLogSearchMatch`, `ActionsJobLogOptions`, and helpers that reuse the `actions-003` log storage/download primitives while grouping lines by step.
- `crates/api/src/routes/actions.rs`: add `GET /api/repos/{owner}/{repo}/actions/runs/{run_id}/jobs/{job_id}/detail` with `q`, `match`, `timestamps`, `raw`, `page`, and `pageSize` query support; preserve existing `/actions/jobs/{job_id}/logs` endpoints.
- `crates/api/migrations/*_actions_job_log_preferences.*.sql`: add only missing additive persistence for per-user log display preferences if no suitable table already exists.
- `crates/api/tests/api_repository_actions_job_log_contract.rs`: seed public/private repositories, runs, sibling jobs, steps with mixed conclusions, searchable log lines, annotations, deleted logs, expired artifacts, and viewer preferences; assert permission filtering, run/job mismatch 404s, step ordering, search match counts, 410-style unavailable envelopes, and option persistence.

**Verification**: focused Rust job-log contract tests, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, and same-env `make test`. Browser smoke is optional in this API-first phase.

---

## Phase 2: Editorial Job Viewer Page - dedicated job route renders the real workspace

**Done**: [x]

**Scope**: Build `/{owner}/{repo}/actions/runs/{run_id}/jobs/{job_id}` as an Editorial job-log workspace. The run left sidebar stays visible, the selected job is highlighted, the main panel shows job title/status/duration, annotations toggle, search controls, options menu, and a step list that can expand into stored log chunks.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed job-log-detail fetch helpers, stable job-view URL builders, and cookie forwarding.
- `web/src/app/[owner]/[repo]/actions/runs/[runId]/jobs/[jobId]/page.tsx`: fetch the Phase 1 contract, preserve repository shell/tab activation, and render not-found/forbidden/unavailable states with standard Editorial primitives.
- `web/src/components/RepositoryActionsJobLogPage.tsx`: new component using `.btn`, `.chip`, `.card`, `.input`, `.list-row`, `.tabs`, `.t-*`, and `var(--*)`; implement the two-column Actions layout from `og-screens-4.jsx`, selected-job sidebar, job header, annotation summary, step rows, and responsive log panel.
- `web/tests/repository-actions-job-log.test.tsx`: cover job header metadata, sibling job navigation, step expand/collapse state, annotations toggle visibility, unavailable-log state, accessible labels, no dead buttons or `href="#"`, and Editorial-token-only styling.
- `web/tests/e2e/repository-actions-job-log.spec.ts`: signed-session browser smoke for a seeded job viewer page and screenshot `ralph/screenshots/build/actions-004-phase2-job-viewer.jpg`.

**Verification**: focused job-viewer Vitest, focused job-viewer Playwright smoke, `make check`, `make test`, and focused same-env `make test-e2e`.

---

## Phase 3: Search, Navigation, and Step Interaction - log inspection controls work

**Done**: [x]

**Scope**: Make the job viewer useful for inspection. Step chevrons expand/collapse chunks, the search input highlights matching lines, previous/next icon buttons jump between matches, result counts stay accurate, copy/permalink line actions work, and live in-progress logs refresh without resetting open steps or scroll position.

**Key changes**:
- `crates/api/src/domain/actions.rs`: extend job-log detail/search helpers with match windowing, step-local result counts, raw/timestamp transforms, stable line anchors, and in-progress polling cursors.
- `crates/api/src/routes/actions.rs`: add or extend job-detail query parameters for selected match and polling cursor while keeping bounded page sizes and standard error envelopes.
- `web/src/components/RepositoryActionsJobLogPage.tsx`: add controlled search state, previous/next result icon buttons with disabled states, highlighted line fragments, sticky current-match marker, copy/permalink buttons per line, expanded-step state keyed by step id, and polling that preserves scroll/open state.
- `web/tests/repository-actions-job-log.test.tsx`: cover search highlight rendering, previous/next wrap or boundary behavior, match count labels, copy/permalink feedback, step collapse preservation across search, and in-progress polling state updates.
- `web/tests/e2e/repository-actions-job-log.spec.ts`: browser smoke for expanding steps, searching a log term, jumping between matches, copying a permalink, and preserving scroll after a refresh.

**Verification**: focused Rust search/poll contract tests, focused job-viewer Vitest, focused Playwright interaction smoke, `make check`, `make test`, and focused same-env `make test-e2e`.

---

## Phase 4: Log Options and Archive Download - every toolbar action is data-backed

**Done**: [ ]

**Scope**: Wire the log options dropdown and archive/download behavior. Users can toggle timestamps, switch raw/rendered log display, copy the job permalink, download a single-job log, and download the run log archive through signed URLs or deterministic local-dev responses. Expired/deleted logs render a 410-style unavailable state with no dead controls.

**Key changes**:
- `crates/api/src/domain/actions.rs`: add preference write/read helpers, single-job archive metadata, run archive lookup, signed download URL generation, expired/deleted-log checks, and audit/request-log metadata where applicable.
- `crates/api/src/routes/actions.rs`: add `PATCH /api/repos/{owner}/{repo}/actions/log-preferences`, `GET /api/repos/{owner}/{repo}/actions/runs/{run_id}/logs/archive`, and any needed job-scoped archive alias; reuse existing job download endpoint when possible.
- `web/src/app/[owner]/[repo]/actions/runs/[runId]/logs/archive/route.ts` and preference proxy routes as needed: forward cookies and preserve Rust error envelopes.
- `web/src/components/RepositoryActionsJobLogPage.tsx`: add an accessible options menu, timestamp/raw toggles, copy job permalink action, single-job download, run-archive download, unavailable/expired disabled states, and success/error feedback.
- `web/src/lib/api-docs.ts` and `/docs/api`: document the job viewer detail endpoint, log preference write, job log download, run archive download, 410 unavailable response, query params, and permissions.
- Tests: cover preference persistence, archive URL shape, deleted/expired behavior, docs anchors, menu keyboard behavior, and browser smoke `ralph/screenshots/build/actions-004-phase4-log-options.jpg`.

**Verification**: focused Rust preferences/archive contract tests, focused docs/job-viewer Vitest, focused Playwright options smoke, `make check`, `make test`, and same-env `make test-e2e`.

---

## Phase 5: Final Job Log Guardrails and QA Handoff - complete actions-004 safely

**Done**: [ ]

**Scope**: Harden the dedicated job log viewer before setting `actions-004.build_pass=true`. Validate permissions, deleted/expired logs, search bounds, keyboard/focus behavior, mobile layout, visual compliance, browser evidence, QA hints, and bookkeeping.

**Key changes**:
- `crates/api/tests/api_repository_actions_job_log_contract.rs`: final matrix for anonymous public reads, private denial, owner private access, run/job mismatch, missing jobs, deleted/expired logs, in-progress polling, preference persistence, archive downloads, and no private path/storage-key leakage.
- `web/tests/repository-actions-job-log.test.tsx` and `web/tests/e2e/repository-actions-job-log.spec.ts`: final dead-control sweep, keyboard traversal through sidebar/search/options/steps, result navigation, mobile layout, no horizontal overflow, and screenshots `ralph/screenshots/build/actions-004-phase5-final-desktop.jpg` and `ralph/screenshots/build/actions-004-phase5-final-mobile.jpg`.
- Mandatory Editorial banned-value scan before commit: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`: append honest notes for real runner execution not being part of this feature, S3 signed-URL/dev-storage limits, polling race cases, very large logs, deleted-log retention, archive expiration, and browser clipboard limitations.
- `prd.json`: set only `actions-004.build_pass` to `true` after every phase passes; leave `qa_pass=false`.
- `build-progress.txt`: append final summary, verification evidence, changed files, and known risks.

**Verification**: focused job-log Rust contract tests, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`, and mandatory Editorial banned-value scan using the local compatible `rg -n -e` form.
