# Structure Outline: code-security-002 Dependabot Vulnerability Alerts

**Ticket**: `code-security-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, `target-docs/content/code-security/concepts/supply-chain-security/about-dependabot-alerts.md`, `target-docs/content/code-security/how-tos/manage-security-alerts/manage-dependabot-alerts/viewing-and-updating-dependabot-alerts.md`, existing Dependency graph contracts from `insights-005`, existing Security shell and policy contracts from `code-security-001`, notifications contracts from `notifications-001`, and PR creation contracts from `prs-003`.
**Date**: 2026-05-05

## Phase 1: Alert API Contract and Persistence - screen-ready list/detail data

**Done**: [x]

**Scope**: Add the repository-owned Dependabot alert read model and API contracts without rendering the page yet. `GET /api/repos/{owner}/{repo}/security/dependabot` should return enabled/disabled state, open/closed counts, filter metadata, selectable alert rows, and privacy-safe empty states. `GET /api/repos/{owner}/{repo}/security/dependabot/{alert_id}` should return detail data, advisory metadata, timeline rows, assignee options, and security-update affordances.

**Key changes**:
- `crates/api/migrations/*_dependabot_alerts.*.sql`: add `dependabot_alerts`, `security_alert_events`, alert assignment joins, and narrow security-update metadata only where existing `dependency_manifests`, `repository_dependencies`, `dependency_packages`, `dependency_advisories`, `repository_security_feature_settings`, `notifications`, and `security_audit_events` are insufficient.
- `crates/api/src/domain/repository_security.rs` or `crates/api/src/domain/dependabot_alerts.rs`: add DTOs such as `DependabotAlertsView`, `DependabotAlertRow`, `DependabotAlertDetail`, `DependabotAlertFilters`, `DependabotAlertTimelineEvent`, `DependabotAlertAssignmentOption`, and `DependabotSecurityUpdateState`.
- `crates/api/src/routes/repositories.rs`: register authenticated list/detail routes, enforce repository read permission plus private outsider 404 privacy, normalize `state`, `q`, `package`, `ecosystem`, `manifest`, `scope`, `severity`, and `sort`, and return standard 422 envelopes for invalid filters.
- Alert generation helper: derive initial alert rows from existing dependency graph rows joined to `dependency_advisories`; preserve stable alert numbers and avoid mock data.
- `crates/api/tests/repository_dependabot_alerts_contract.rs`: seed vulnerable and safe dependencies, enabled/disabled feature settings, permissions, advisories, assignments, and verify list/detail DTO shape, disabled state, filter/sort behavior, private repository privacy, and no token/session/env leakage.
- `web/src/lib/api.ts` and server fetch helpers: add typed Dependabot list/detail DTOs without adding client-side auth.

**Verification**: focused Rust contract tests against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Dependabot Alerts List - filters, selection, and disabled state

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/security/dependabot` inside `RepositorySecurityShell` with Dependabot selected. The page should render Open/Closed tabs, search, filter menus, sort menu, selectable alert rows, disabled callout, settings link, and concrete alert detail links from the Phase 1 API.

**Key changes**:
- `web/src/app/[owner]/[repo]/security/dependabot/page.tsx`: server-fetch repository metadata and Dependabot list data, preserve query params, and render unavailable/forbidden states without leaking private counts.
- `web/src/components/RepositoryDependabotAlertsPage.tsx`: render the header, Give feedback link, alert settings menu, tab counts, search bar, package/ecosystem/manifest/scope/severity filters, sort menu defaulting to Most important, selectable rows, disabled state with settings link, and empty states using Editorial primitives/tokens only.
- `web/src/components/RepositoryDependabotAlertFilters.tsx`: client URL controls for search/filter/sort, outside-click/Escape handling, accessible selected-state affordances, and Clear/Apply actions.
- `web/src/lib/navigation.ts`: add Dependabot list/detail/settings href helpers that safely encode owners, repositories, packages, manifests, and alert IDs.
- `web/tests/repository-dependabot-alerts-page.test.tsx`: cover active Security sidebar state, disabled callout, Open/Closed tabs, filters, sort, row selection, concrete row/detail links, no `href="#"`, no inert handlers, no unsafe HTML, mobile wrapping, and Editorial banned-value guardrails.
- `web/tests/e2e/repository-dependabot.spec.ts`: focused signed-session browser smoke for the default list, filters, row navigation, disabled state, no dead controls, and screenshot `ralph/screenshots/build/code-security-002-phase2-alerts-list.jpg`.

**Verification**: focused Vitest and focused Playwright smoke, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Alert Detail and Single-Alert Triage - dismiss, reopen, assign, and timeline

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/security/dependabot/{alert_id}` plus single-alert mutations. Maintainers can dismiss open alerts with reason/comment, reopen dismissed alerts, assign users/teams, and see an audit-backed timeline. Readers can view detail data but cannot mutate.

**Key changes**:
- `PATCH /api/repos/{owner}/{repo}/security/dependabot/{alert_id}`: support `dismiss`, `reopen`, and assignment changes with maintainer permission, state-transition validation, required dismissal reason, optional bounded comment, closed/fixed reopening rules, and standard conflict/error envelopes.
- `crates/api/src/domain/dependabot_alerts.rs`: write `dependabot_alerts` state changes, `security_alert_events`, `security_audit_events`, and `notifications` updates atomically; redact private metadata in audit payloads.
- `web/src/app/[owner]/[repo]/security/dependabot/[alertId]/page.tsx` and mutation proxy route or server action: fetch detail and forward authenticated mutations to Rust.
- `web/src/components/RepositoryDependabotAlertDetailPage.tsx`: render vulnerable dependency, advisory, affected/fixed versions, tags, manifest link, assignee selector, timeline, dismiss dropdown with reason/comment, reopen action, and reader/maintainer states with Editorial controls only.
- Extend Rust, Vitest, and Playwright coverage for detail rendering, dismiss validation, dismissed timeline rows, reopen, assignments, unauthorized reader mutations, fixed alert non-reopen behavior, notification writes, audit writes, no dead controls, and screenshot `ralph/screenshots/build/code-security-002-phase3-alert-detail.jpg`.

**Verification**: focused Rust mutation contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 4: Bulk Triage and Security Update PR CTA - list actions become real writes

**Done**: [ ]

**Scope**: Make list-row selection useful. Maintainers can bulk dismiss/reopen selected alerts. Supported open alerts expose a security-update PR CTA that creates or links a real pull request using existing PR infrastructure and records notifications/audit events.

**Key changes**:
- `POST /api/repos/{owner}/{repo}/security/dependabot/bulk`: validate selected alert IDs, shared state constraints, dismissal reason/comment, maintainer permission, and return per-alert results without partially hidden private data.
- `POST /api/repos/{owner}/{repo}/security/dependabot/{alert_id}/security-update`: create or reuse a security update branch/PR through existing repository file and `pull_requests` contracts where the dependency ecosystem has a deterministic manifest edit path; otherwise return an unsupported state with truthful UI copy.
- `web/src/components/RepositoryDependabotBulkActions.tsx`: selection summary, Select all visible, Dismiss selected dropdown, Reopen selected action, pending/success/error states, and server-confirmed refresh behavior.
- `web/src/components/RepositoryDependabotSecurityUpdateButton.tsx`: detail-page CTA with pending/success/error states, linked PR destination, unsupported state, and no fake PR creation.
- Extend tests for bulk dismiss/reopen, mixed-result validation, security update PR creation/reuse, notification fanout, audit redaction, disabled/unsupported CTA states, and mobile no-overflow.

**Verification**: focused Rust bulk/security-update contracts, focused Vitest, focused Playwright smoke with screenshot `ralph/screenshots/build/code-security-002-phase4-bulk-security-update.jpg`, mandatory Editorial banned-value scan, then `make check && make test`; run full `make test-e2e` when the local migration state allows it.

---

## Phase 5: API Docs, Edge Cases, Browser Evidence, QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `code-security-002` only after list, detail, single triage, bulk triage, security-update PR flow, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document Dependabot list/detail/mutation/bulk/security-update endpoints with auth/privacy gates, filters, sort values, disabled states, state transitions, security-update behavior, notifications, audit events, and no-secret error envelopes.
- Final Rust tests: cover disabled feature settings, archived repositories, malformed filters, stale alert state conflicts, long package/manifest names, multiple ecosystems, fixed vs dismissed alerts, hidden private repositories, notification updates, audit redaction, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard focus order through filters/rows/actions, alert detail controls, bulk action validation, semantic severity chips, long text wrapping, mobile no-overflow, no unsafe HTML, no `href="#"`, no inert click handlers, and Editorial token compliance.
- `web/tests/e2e/repository-dependabot.spec.ts`: full signed-session browser sweep for list filters, row selection, bulk dismiss/reopen, alert detail, single dismiss/reopen, assignment, security update PR CTA, disabled state, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/code-security-002/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `code-security-002.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/code-security-002-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
