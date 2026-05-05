# Structure Outline: code-security-001 Repository Security Overview and Policy

**Ticket**: `code-security-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/wf-repo.jsx`, current repository shell in `web/src/components/RepositoryShell.tsx`, current `/[owner]/[repo]/security` placeholder, repository file/blob contracts in `crates/api/src/domain/repositories.rs`, markdown rendering in `crates/api/src/domain/markdown.rs`, git materialization in `crates/api/src/domain/git_transport.rs`, and security policy/advisory references under `target-docs/content/code-security/` and `target-docs/content/rest/security-advisories/`.
**Date**: 2026-05-05

## Phase 1: Security Overview API Contract - policy, advisories, feature cards, and privacy gates

**Done**: [x]

**Scope**: Add the screen-ready Rust/Postgres read contract for `/api/repos/{owner}/{repo}/security`. Readers can see published policy content and public advisory summaries, while alert counts and private repository metadata remain permission-gated.

**Key changes**:
- `crates/api/migrations/`: add or extend tables for `repository_security_feature_settings`, `repository_security_policies`, `repository_security_advisories`, and `security_audit_events` only where existing tables are insufficient. Store policy source path/ref/blob metadata, rendered Markdown cache metadata, feature status summaries, advisory severity/status/package rows, and redacted audit fields.
- `crates/api/src/domain/repository_security.rs`: introduce DTOs such as `RepositorySecurityOverview`, `SecurityPolicySummary`, `SecurityFeatureCard`, `RepositorySecurityAdvisorySummary`, `SecurityViewer`, and structured unavailable/forbidden states.
- `crates/api/src/routes/repositories.rs` or a dedicated security route module: add authenticated `GET /api/repos/{owner}/{repo}/security` with repository read permission, anonymous 401, private outsider 404, reader-safe public policy data, maintainer-only setup/edit affordances, and no leaked private alert counts.
- `crates/api/tests/repository_security_policy_contract.rs`: seed repositories with and without `SECURITY.md`, feature settings, published/draft advisories, private alerts, permissions, and verify DTO shape, markdown rendering/sanitization, privacy, deterministic ordering, and no stack/session/token leakage.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed fetch helpers for the overview contract without adding client-side auth.

**Verification**: focused `repository_security_policy_contract` against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Security Overview Page - `/security` becomes the Security and quality workspace

**Done**: [x]

**Scope**: Replace the placeholder with the Editorial Security and quality page backed by Phase 1. The page should show the left security sidebar, rendered policy preview, advisory rows, and feature cards with concrete navigation/actions.

**Key changes**:
- `web/src/components/RepositorySecurityShell.tsx`: new left-sidebar wrapper inside `RepositoryShell`, with groups for Overview, Findings, Dependabot, Code scanning, Secret scanning, Reporting, Security policy, and Advisories. Active state must use `--accent` and existing `.tabs`/`.list-row`/`.chip` primitives.
- `web/src/components/RepositorySecurityOverviewPage.tsx`: render policy preview or missing-policy state, maintainer setup/edit CTA, recent published advisories with severity/status chips, and Dependabot/code scanning/secret scanning cards using server-confirmed feature metadata.
- `web/src/app/[owner]/[repo]/security/page.tsx`: fetch the overview contract server-side and render forbidden/unavailable states without exposing private counts.
- `web/src/lib/navigation.ts`: add security-section URL helpers if needed for sidebar links and policy/advisory destinations.
- `web/tests/repository-security-overview-page.test.tsx`: cover active repository Security tab, security sidebar state, policy preview, missing-policy reader/maintainer variants, advisory rows, feature cards, no dead links/buttons, no unsafe HTML, and Editorial banned-value guardrails.

**Verification**: focused Vitest for the page, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`. Save a browser screenshot if seeded data already supports the overview.

---

## Phase 3: Security Policy Page and Markdown Reader - `/security/policy` exposes anchors, links, and file actions

**Done**: [ ]

**Scope**: Add the dedicated policy reader page. It must render sanitized Markdown with heading anchors, `mailto:` links, file action menu destinations, and clear empty states for maintainers vs readers.

**Key changes**:
- `crates/api/src/domain/repository_security.rs`: add `RepositorySecurityPolicyView` or extend the overview helper to return full policy markdown/html, heading outline, source blob/raw/history/edit hrefs, latest commit metadata, and viewer edit permissions.
- `GET /api/repos/{owner}/{repo}/security/policy`: return full policy content from the default branch `SECURITY.md` or configured policy path, including rendered Markdown from the existing Rust renderer and no raw private metadata leaks.
- `web/src/app/[owner]/[repo]/security/policy/page.tsx`: server-render the policy page through `RepositorySecurityShell`.
- `web/src/components/RepositorySecurityPolicyPage.tsx`: render the Markdown body, generated heading anchors, email links, source path/ref metadata, file action menu links to blob/raw/history, maintainer Start setup/Edit policy CTA, and reader missing-policy message.
- `web/tests/repository-security-policy-page.test.tsx`: assert heading anchors, safe mailto/external links, file action hrefs, empty states, maintainer CTA visibility, no `dangerouslySetInnerHTML` bypass beyond sanitized `MarkdownBody`, and responsive no-overflow behavior.

**Verification**: focused Rust policy contract, focused Vitest, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke should save `ralph/screenshots/build/code-security-001-phase3-policy.jpg`.

---

## Phase 4: Policy Create/Edit Flow - maintainer writes update the real repository file and audit trail

**Done**: [ ]

**Scope**: Wire Start setup/Edit policy to a real write flow that creates or updates `SECURITY.md`, records a commit, updates the default branch ref, refreshes `repository_security_policies`, and makes Code/blob/raw views reflect the new file.

**Key changes**:
- `POST /api/repos/{owner}/{repo}/security/policy` and `PATCH /api/repos/{owner}/{repo}/security/policy`: validate maintainer permission, branch/ref freshness, path constraints, non-empty Markdown, commit message, and optional propose-change fallback for users without direct push permission.
- `crates/api/src/domain/repository_security.rs`: implement policy write helpers that reuse existing git/file materialization patterns instead of a parallel fake store; update `repository_files`, `git_objects`, `commits`, `repository_git_refs`, `repository_security_policies`, `repository_activity_events`, and `security_audit_events` atomically.
- `web/src/app/[owner]/[repo]/security/policy/actions/route.ts` or server actions: forward authenticated policy mutations to Rust, return standard error envelopes, and never store client-only optimistic state.
- `web/src/components/RepositorySecurityPolicyEditor.tsx`: textarea/editor form with template starter content, commit-message field, preview tab using existing Markdown rendering, save/propose-change button, inline validation/errors, and reload from server-confirmed state.
- Extend Rust, Vitest, and Playwright coverage for create, update, stale ref conflict, unauthorized reader, archived repository, generated commit/blob/raw visibility, audit redaction, form validation, preview, and no dead controls.

**Verification**: focused Rust mutation contract, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/code-security-001-phase4-policy-edit.jpg`, mandatory Editorial banned-value scan, then `make check && make test`; run full `make test-e2e` when the local migration state allows it.

---

## Phase 5: Guardrails, API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `code-security-001` only after overview, policy reader, policy write flow, docs, screenshots, and QA handoff are verified. Do not implement full Dependabot alert triage, code scanning findings, secret scanning findings, or advisory authoring here; those belong to later code-security features.

**Key changes**:
- `web/src/lib/api-docs.ts`: document `GET /api/repos/{owner}/{repo}/security`, `GET /api/repos/{owner}/{repo}/security/policy`, and policy create/update endpoints with auth/privacy, markdown sanitization, commit/ref behavior, validation errors, and no-secret error envelopes.
- Final Rust tests: cover published vs draft advisories, private alert count redaction, policy path precedence, malformed Markdown safety, branch/ref conflicts, archived writes, concurrent policy updates, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard navigation through the security sidebar, policy actions, empty states, advisory rows, feature card links, long policy text wrapping, mobile no-overflow, and Editorial token compliance.
- `web/tests/e2e/repository-security.spec.ts`: signed-in browser smoke for overview, policy page, maintainer create/edit flow, blob/raw reflection, reader missing-policy state, and mobile layout; save final screenshots under `ralph/screenshots/build/code-security-001-final-*.jpg`.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/code-security-001/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `code-security-001.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when available, browser smoke screenshots under `ralph/screenshots/build/`, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan with zero matches.
