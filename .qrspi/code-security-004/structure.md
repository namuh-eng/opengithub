# Structure Outline: code-security-004 Secret Scanning Alerts and Push Protection

**Ticket**: `code-security-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, `target-docs/content/code-security/concepts/secret-security/about-secret-scanning.md`, `target-docs/content/code-security/concepts/secret-security/about-alerts.md`, `target-docs/content/code-security/concepts/secret-security/about-validity-checks.md`, `target-docs/content/code-security/concepts/secret-security/about-push-protection.md`, `target-docs/content/code-security/concepts/secret-security/about-bypass-requests-for-push-protection.md`, `target-docs/content/code-security/how-tos/manage-security-alerts/manage-secret-scanning-alerts/viewing-alerts.md`, `target-docs/content/code-security/how-tos/manage-security-alerts/manage-secret-scanning-alerts/resolving-alerts.md`, `target-docs/content/code-security/how-tos/secure-your-secrets/detect-secret-leaks/enabling-secret-scanning-for-your-repository.md`, `target-docs/content/code-security/how-tos/secure-your-secrets/prevent-future-leaks/enabling-push-protection-for-your-repository.md`, existing Security shell and feature-setting contracts from `code-security-001`, existing alert/timeline/audit patterns from `code-security-002` and `code-security-003`, Git push contracts from `git-001`, notification contracts from `notifications-001`, and repository settings policy contracts from `settings-004`.
**Date**: 2026-05-05

## Phase 1: Secret Scanning API Contract and Persistence - redacted list/detail data

**Done**: [x]

**Scope**: Add repository-owned secret scanning persistence and read APIs without rendering the page yet. `GET /api/repos/{owner}/{repo}/security/secret-scanning` should return enabled/disabled state, provider/default and generic tab counts, filter metadata, selectable alert rows, push-protection summary, and privacy-safe empty states. `GET /api/repos/{owner}/{repo}/security/secret-scanning/{alert_id}` should return redacted evidence, file/commit location, validity, resolution state, bypass metadata, assignment options, timeline rows, and viewer affordances. Plaintext secret values must never be stored, returned, logged, or included in audit payloads.

**Key changes**:
- `crates/api/migrations/*_secret_scanning_alerts.*.sql`: add `secret_scanning_patterns`, `secret_scanning_alerts`, `secret_scanning_alert_locations`, `secret_scanning_alert_assignees`, `secret_scanning_validity_checks`, `push_protection_bypasses`, and narrow indexes/joins only where existing `repository_security_feature_settings`, `repository_files`, `repository_git_refs`, `commits`, `users`, `teams`, `notifications`, `security_alert_events`, and `security_audit_events` are insufficient.
- `crates/api/src/domain/secret_scanning_alerts.rs`: define DTOs such as `SecretScanningAlertsView`, `SecretScanningAlertRow`, `SecretScanningAlertDetail`, `SecretScanningFilters`, `SecretScanningLocation`, `SecretScanningTimelineEvent`, `SecretScanningAssignmentOption`, `SecretScanningValidityState`, and `PushProtectionSummary`.
- `crates/api/src/routes/repositories.rs`: register authenticated list/detail routes, enforce repository read permission plus private outsider 404 privacy, normalize `state`, `q`, `provider`, `secret_type`, `validity`, `resolution`, `bypassed`, `team`, `topic`, and `sort`, and return standard 422 envelopes for invalid filters.
- Alert materialization helper: derive stable fingerprints from provider pattern, commit/blob identity, path/line, and a keyed hash of the secret bytes; persist only fingerprints plus redacted snippets with bounded context.
- `crates/api/tests/repository_secret_scanning_alerts_contract.rs`: seed enabled/disabled settings, provider and generic patterns, redacted alert locations, validity checks, bypass rows, permissions, assignments, and verify list/detail DTO shape, filters, disabled state, private repository privacy, invalid filters, and no plaintext secret/session/env leakage.
- `web/src/lib/api.ts` and server fetch helpers: add typed secret scanning list/detail DTOs without adding client-side auth.

**Verification**: focused Rust contract tests against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Secret Scanning Alerts List - filters, tabs, and disabled state

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/security/secret-scanning` inside `RepositorySecurityShell` with Secret scanning selected. The page should render the disabled callout, provider/default and generic results tabs, query input, filter menus, sort menu, selectable rows, push-protection/bypass summary, and concrete alert detail/file/settings links from the Phase 1 API.

**Key changes**:
- `web/src/app/[owner]/[repo]/security/secret-scanning/page.tsx`: server-fetch repository metadata and secret scanning list data, preserve query params, and render unavailable/forbidden states without leaking private counts.
- `web/src/components/RepositorySecretScanningAlertsPage.tsx`: render Secret scanning alerts heading, enable/settings actions, disabled explanatory state, provider/default and generic result tabs, query input, provider/secret-type/validity/resolution/bypassed/state/team/topic filters, sort menu, selectable alert rows, redacted secret type, provider badge, path/line, validity chip, assignee, bypassed marker, timestamps, and empty states using Editorial primitives/tokens only.
- `web/src/components/RepositorySecretScanningAlertFilters.tsx`: client URL controls for query/filter/sort, outside-click/Escape handling, accessible selected-state affordances, and Clear/Apply actions.
- `web/src/lib/navigation.ts`: add secret scanning list/detail/settings/file/commit href helpers that safely encode owners, repositories, refs, paths, lines, and alert IDs.
- `web/tests/repository-secret-scanning-alerts-page.test.tsx`: cover active Security sidebar state, disabled callout, result tabs, filters, sort, row selection, redacted evidence display, concrete row/detail/file/commit links, no `href="#"`, no inert handlers, no unsafe HTML, mobile wrapping, and Editorial banned-value guardrails.
- `web/tests/e2e/repository-secret-scanning.spec.ts`: focused signed-session browser smoke for the default list, filters, row navigation, disabled state, no dead controls, and screenshot `ralph/screenshots/build/code-security-004-phase2-alerts-list.jpg`.

**Verification**: focused Vitest and focused Playwright smoke, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Alert Detail and Single-Alert Triage - resolve, reopen, assign, timeline

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/security/secret-scanning/{alert_id}` plus single-alert mutations. Maintainers can resolve open alerts as revoked, false positive, used in tests, or won't fix; reopen resolved alerts; assign users/teams; and inspect a redacted audit-backed timeline. Readers can view permitted redacted detail data but cannot mutate.

**Key changes**:
- `PATCH /api/repos/{owner}/{repo}/security/secret-scanning/{alert_id}`: support `resolve`, `reopen`, validity updates where allowed, and assignment changes with maintainer permission, state-transition validation, required resolution reason, optional bounded comment, stale state conflicts, and standard no-secret error envelopes.
- `crates/api/src/domain/secret_scanning_alerts.rs`: write alert state changes, validity rows, assignment joins, `security_alert_events`, `security_audit_events`, and `notifications` atomically; ensure audit/event payloads carry only redacted snippets and fingerprints.
- `web/src/app/[owner]/[repo]/security/secret-scanning/[alertId]/page.tsx` and mutation proxy route or server action: fetch detail and forward authenticated mutations to Rust.
- `web/src/components/RepositorySecretScanningAlertDetailPage.tsx`: render redacted evidence, file breadcrumbs and line anchors, commit metadata, provider/secret type, validity state, resolution form, assignee selector, bypass section, timeline, reopen action, and reader/maintainer states with Editorial controls only.
- Extend Rust, Vitest, and Playwright coverage for detail rendering, resolution validation, resolved timeline rows, reopen, assignments, unauthorized reader mutations, validity metadata, notification writes, audit redaction, no dead controls, and screenshot `ralph/screenshots/build/code-security-004-phase3-alert-detail.jpg`.

**Verification**: focused Rust mutation contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 4: Secret Detection and Push Protection Integration - real indexing and bypass outcomes

**Done**: [x]

**Scope**: Make secret scanning real by indexing committed blobs and enforcing push protection during Rust Git pushes. When secret scanning is enabled, existing blobs and new pushed commits are scanned against provider/default plus generic patterns; matching secrets create redacted alerts. Push protection blocks or warns on protected matches; allowed bypasses require a reason, create bypass rows, alert events, audit events, and notifications.

**Key changes**:
- `crates/api/src/domain/secret_scanning_detection.rs`: pattern registry, bounded blob scanning, binary/large-file skips, path exclusions, generic/provider classification, line/context redaction, stable fingerprinting, and validity-check placeholders without network calls unless a provider verifier is explicitly configured.
- Git push integration in the existing `git-001` smart HTTP receive path: scan incoming commits before accepting refs when push protection is enabled, return a structured block/warn response for protected patterns, accept authorized bypass payloads only with a valid reason, and never echo plaintext secret matches.
- Repository backfill helper/job: scan existing `repository_files`/blob snapshots when the feature is enabled, de-duplicate alerts by fingerprint/location, mark alerts resolved/fixed when a latest scan no longer finds them only where that state is supported, and record scan status/freshness.
- `push_protection_bypasses`: persist actor, commit/ref/path metadata, redacted snippet, reason, expiry/review status if applicable, and links to created alerts/events.
- Extend Rust contract tests for provider/default/generic detection, redaction, disabled feature no-op behavior, archived repository behavior, binary/large-file skips, bypass-required validation, accepted bypass writes, notification fanout, audit redaction, and no plaintext leakage in logs/errors.
- Extend Vitest/Playwright coverage for push-protection summary/bypassed markers on the list/detail pages and screenshot `ralph/screenshots/build/code-security-004-phase4-push-protection.jpg`.

**Verification**: focused Rust detection and git-push contracts, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`; run full `make test-e2e` when the local migration state allows it.

---

## Phase 5: API Docs, Edge Cases, Browser Evidence, QA Handoff, and Build Pass

**Done**: [x]

**Scope**: Finish `code-security-004` only after list, detail, triage, scanning/backfill, push protection, bypass outcomes, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document secret scanning list/detail/mutation and push-protection/bypass endpoints with auth/privacy gates, filters, sort values, disabled states, resolution transitions, validity metadata, push-protection responses, notification fanout, audit events, and no-secret error envelopes.
- Final Rust tests: cover disabled feature settings, archived repositories, malformed filters, stale alert state conflicts, long paths/type names, multiple providers/patterns, generic results, resolved vs reopened alerts, hidden private repositories, bypass outcomes, notification updates, audit redaction, and absence of plaintext secrets/session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard focus order through filters/rows/actions, alert detail controls, resolution form validation, semantic validity/resolution chips, long path/type wrapping, mobile no-overflow, no unsafe HTML, no `href="#"`, no inert click handlers, and Editorial token compliance.
- `web/tests/e2e/repository-secret-scanning.spec.ts`: full signed-session browser sweep for list filters, row selection, alert detail, resolve/reopen, assignment, bypass marker review, disabled state, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/code-security-004/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `code-security-004.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/code-security-004-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
