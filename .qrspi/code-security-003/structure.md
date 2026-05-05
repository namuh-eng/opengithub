# Structure Outline: code-security-003 Code Scanning Alerts

**Ticket**: `code-security-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, `target-docs/content/code-security/concepts/code-scanning/about-code-scanning.md`, `target-docs/content/code-security/concepts/code-scanning/about-code-scanning-alerts.md`, `target-docs/content/code-security/concepts/code-scanning/sarif-files.md`, `target-docs/content/code-security/how-tos/find-and-fix-code-vulnerabilities/integrate-with-existing-tools/uploading-a-sarif-file-to-github.md`, `target-docs/content/code-security/how-tos/manage-security-alerts/manage-code-scanning-alerts/assessing-code-scanning-alerts-for-your-repository.md`, `target-docs/content/code-security/how-tos/manage-security-alerts/manage-code-scanning-alerts/resolving-code-scanning-alerts.md`, `target-docs/content/code-security/how-tos/manage-security-alerts/manage-code-scanning-alerts/linking-code-scanning-alerts-to-github-issues.md`, existing Security shell and policy contracts from `code-security-001`, existing Dependabot alert patterns from `code-security-002`, Actions/check-run contracts from `actions-005`, PR annotation expectations from `prs-006`, and issue creation/linking contracts from `issues-004`.
**Date**: 2026-05-05

## Phase 1: Code Scanning API Contract and Persistence - screen-ready list/detail data

**Done**: [x]

**Scope**: Add repository-owned code scanning persistence and read APIs without rendering the page yet. `GET /api/repos/{owner}/{repo}/security/code-scanning` should return enabled/disabled state, open/closed counts, filter metadata, SARIF tool/status summaries, and selectable alert rows. `GET /api/repos/{owner}/{repo}/security/code-scanning/{alert_id}` should return detail data, code location/snippet metadata, rule/help/remediation content, timeline rows, assignment options, linked issue state, and reader/maintainer affordances.

**Key changes**:
- `crates/api/migrations/*_code_scanning_alerts.*.sql`: add `code_scanning_runs`, `code_scanning_alerts`, `code_scanning_alert_instances`, `code_scanning_alert_assignees`, `code_scanning_sarif_uploads`, `security_alert_events`, and narrow issue/check-run links only where existing `repository_security_feature_settings`, `workflow_runs`, `check_runs`, `issues`, `repository_files`, `notifications`, and `security_audit_events` are insufficient.
- `crates/api/src/domain/code_scanning_alerts.rs`: define DTOs such as `CodeScanningAlertsView`, `CodeScanningAlertRow`, `CodeScanningAlertDetail`, `CodeScanningFilters`, `CodeScanningLocation`, `CodeScanningTimelineEvent`, `CodeScanningAssignmentOption`, `CodeScanningLinkedIssueState`, and `CodeScanningToolStatus`.
- `crates/api/src/routes/repositories.rs`: register authenticated list/detail routes, enforce repository read plus private outsider 404 privacy, enforce write/admin permission for summary list if the final permission model requires it, normalize `state`, `q`, `severity`, `security_severity`, `tool`, `branch`, `ref`, `tag`, `application_code`, `sort`, and return standard 422 envelopes for invalid filters.
- Alert materialization helper: normalize seeded SARIF-like result rows by `(rule_id, location path/start line, fingerprint, ref)` so repeated uploads update existing alerts instead of duplicating them.
- `crates/api/tests/repository_code_scanning_alerts_contract.rs`: seed enabled/disabled settings, SARIF runs, tools, rules, alert instances, refs, repository files, permissions, assignments, and verify list/detail DTO shape, filters, disabled state, private repository privacy, invalid filters, and no token/session/env leakage.
- `web/src/lib/api.ts` and server fetch helpers: add typed code scanning list/detail DTOs without adding client-side auth.

**Verification**: focused Rust contract tests against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Code Scanning Alerts List - filters, disabled state, and selection

**Done**: [ ]

**Scope**: Implement `/{owner}/{repo}/security/code-scanning` inside `RepositorySecurityShell` with Code scanning selected. The page should render disabled/not-enabled state, Open/Closed tabs, search, filter menus, sort menu, selectable alert rows, alert-settings/setup links, and concrete detail/file links from the Phase 1 API.

**Key changes**:
- `web/src/app/[owner]/[repo]/security/code-scanning/page.tsx`: server-fetch repository metadata and code scanning list data, preserve query params, and render unavailable/forbidden states without leaking disabled historical counts.
- `web/src/components/RepositoryCodeScanningAlertsPage.tsx`: render the header, setup/settings actions, disabled callout with Enable code scanning link, tab counts, search box, severity/tool/branch/ref/tag/application-code filters, sort menu, selectable rows, linked issue chips, file path/line links, branch/default-branch badges, and empty states using Editorial primitives/tokens only.
- `web/src/components/RepositoryCodeScanningAlertFilters.tsx`: client URL controls for search/filter/sort, outside-click/Escape handling, accessible selected-state affordances, and Clear/Apply actions.
- `web/src/lib/navigation.ts`: add code scanning list/detail/setup/settings/file/issue href helpers that safely encode owners, repositories, refs, paths, lines, and alert IDs.
- `web/tests/repository-code-scanning-alerts-page.test.tsx`: cover active Security sidebar state, disabled callout, Open/Closed tabs, filters, sort, row selection, concrete row/detail/file/issue links, no `href="#"`, no inert handlers, no unsafe HTML, mobile wrapping, and Editorial banned-value guardrails.
- `web/tests/e2e/repository-code-scanning.spec.ts`: focused signed-session browser smoke for the default list, filters, row navigation, disabled state, no dead controls, and screenshot `ralph/screenshots/build/code-security-003-phase2-alerts-list.jpg`.

**Verification**: focused Vitest and focused Playwright smoke, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Alert Detail and Single-Alert Triage - dismiss, reopen, assign, link issue

**Done**: [ ]

**Scope**: Implement `/{owner}/{repo}/security/code-scanning/{alert_id}` plus single-alert mutations. Maintainers can dismiss open alerts with reason/comment, reopen dismissed alerts, assign users/teams, and create or link an issue. Readers can view permitted PR annotations/detail data but cannot mutate.

**Key changes**:
- `PATCH /api/repos/{owner}/{repo}/security/code-scanning/{alert_id}`: support `dismiss`, `reopen`, assignment changes, and linked-issue changes with maintainer permission, state-transition validation, required dismissal reason, optional bounded comment, stale state conflicts, and standard no-secret error envelopes.
- `POST /api/repos/{owner}/{repo}/security/code-scanning/{alert_id}/issue`: create or link an issue through the existing issue contract, prefill the title/body from alert rule/location/remediation data, avoid duplicate linked issues, and record timeline/audit/notification events.
- `crates/api/src/domain/code_scanning_alerts.rs`: write alert state changes, assignment joins, `security_alert_events`, `security_audit_events`, `notifications`, and issue links atomically; redact private source snippets and environment metadata in audit payloads.
- `web/src/app/[owner]/[repo]/security/code-scanning/[alertId]/page.tsx` and mutation proxy route or server action: fetch detail and forward authenticated mutations to Rust.
- `web/src/components/RepositoryCodeScanningAlertDetailPage.tsx`: render code snippet with path breadcrumbs and line anchors, rule title/message, severity/security-severity chips, Show paths disclosure, Show more remediation guidance, assignee selector, timeline, dismiss dropdown, reopen action, linked issue actions, and reader/maintainer states with Editorial controls only.
- Extend Rust, Vitest, and Playwright coverage for detail rendering, dismiss validation, dismissed timeline rows, reopen, assignments, issue creation/linking, unauthorized reader mutations, notification writes, audit redaction, no dead controls, and screenshot `ralph/screenshots/build/code-security-003-phase3-alert-detail.jpg`.

**Verification**: focused Rust mutation contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 4: SARIF Upload and PR Annotation Integration - real ingestion path

**Done**: [ ]

**Scope**: Add the ingestion path that makes code scanning alerts real. Actions or REST uploads can submit SARIF, the API stores upload metadata/artifacts, normalizes alerts by fingerprint/location/rule, updates check-run/PR annotations where applicable, and exposes upload/tool status to the list/detail pages.

**Key changes**:
- `POST /api/repos/{owner}/{repo}/code-scanning/sarifs`: accept bounded SARIF JSON uploads with token/session auth, validate repository write permission, record upload metadata and artifact storage details, reject malformed/oversized payloads with 422/413 envelopes, and enqueue or synchronously normalize small files for MVP.
- `crates/api/src/domain/code_scanning_sarif.rs`: parse SARIF runs/results/rules/locations/fingerprints, normalize severity and security severity, resolve repository files/refs/commits, de-duplicate open alerts, mark fixed-by-new-analysis alerts when absent from latest run, and persist `code_scanning_runs`, `code_scanning_alerts`, and `code_scanning_alert_instances`.
- Check-run/PR integration: update existing `check_runs`/annotation tables for SARIF results attached to a workflow run or head SHA; expose reader-safe PR annotations without bypassing repository permissions.
- `web/src/components/RepositoryCodeScanningUploadStatus.tsx` or list-page status section: show latest SARIF upload/tool status, commit/ref, processed/error counts, and concrete Actions/check-run destinations using Editorial primitives only.
- Extend Rust contract tests for SARIF upload validation, malformed payloads, stable fingerprint updates, fixed alert transitions, PR/check annotations, private repository privacy, S3/local artifact metadata redaction, and no env/secret leakage.
- Extend Vitest/Playwright coverage for upload status, disabled/error upload states, PR/check links, no dead controls, and screenshot `ralph/screenshots/build/code-security-003-phase4-sarif-upload-status.jpg`.

**Verification**: focused Rust SARIF ingestion contracts, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`; run full `make test-e2e` when the local migration state allows it.

---

## Phase 5: API Docs, Edge Cases, Browser Evidence, QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `code-security-003` only after list, detail, triage, SARIF ingestion, PR annotations, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document code scanning list/detail/mutation/issue-link/SARIF upload endpoints with auth/privacy gates, filters, sort values, disabled states, SARIF upload limits, state transitions, PR/check annotations, notification fanout, audit events, and no-secret error envelopes.
- Final Rust tests: cover disabled feature settings, archived repositories, malformed filters, stale alert state conflicts, long paths/rule names, multiple tools, multiple branches/refs/tags, missing source snippets, fixed vs dismissed alerts, hidden private repositories, SARIF parse failures, notification updates, audit redaction, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard focus order through filters/rows/actions, alert detail controls, issue-link actions, semantic severity chips, long path/rule wrapping, mobile no-overflow, no unsafe HTML, no `href="#"`, no inert click handlers, and Editorial token compliance.
- `web/tests/e2e/repository-code-scanning.spec.ts`: full signed-session browser sweep for list filters, row selection, alert detail, single dismiss/reopen, assignment, linked issue flow, SARIF upload status, disabled state, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/code-security-003/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `code-security-003.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/code-security-003-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
