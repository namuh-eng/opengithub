# Structure Outline: code-security-005 Repository Security Advisories

**Ticket**: `code-security-005`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, `target-docs/content/code-security/concepts/vulnerability-reporting-and-management/about-repository-security-advisories.md`, `target-docs/content/code-security/how-tos/report-and-fix-vulnerabilities/fix-reported-vulnerabilities/creating-a-repository-security-advisory.md`, `editing-a-repository-security-advisory.md`, `publishing-a-repository-security-advisory.md`, `adding-a-collaborator-to-a-repository-security-advisory.md`, `target-docs/content/code-security/reference/permissions/permission-levels-for-repository-security-advisories.md`, `target-docs/content/rest/security-advisories/repository-advisories.md`, existing Security shell and overview contracts from `code-security-001`, package/dependency contracts from `packages-001` and `code-security-002`, notification contracts from `notifications-001`, and Markdown/security audit patterns from `code-security-001` through `code-security-004`.
**Date**: 2026-05-05

## Phase 1: Advisory API Contract and Persistence - list/detail data with privacy gates

**Done**: [x]

**Scope**: Expand the existing `repository_security_advisories` summary store into a repository-owned advisory read model and API contract without rendering the pages yet. `GET /api/repos/{owner}/{repo}/security/advisories` should return published advisories for readers and draft plus published rows for authorized maintainers. `GET /api/repos/{owner}/{repo}/security/advisories/{ghsa_id}` should return full metadata, sanitized Markdown, CVSS/CVE/CWE/package details, credits, collaborators, timeline, and viewer affordances.

**Key changes**:
- `crates/api/migrations/*_repository_security_advisory_authoring.*.sql`: add advisory detail columns/tables only where the current overview table is insufficient: CVE, CVSS vector/score/metrics, CWE rows, affected/patched version ranges, Markdown details, credits, collaborators, timeline/events, notification links, dependency advisory feed linkage, and narrow indexes by repository/status/severity/identifier.
- `crates/api/src/domain/repository_security_advisories.rs` or `repository_security.rs`: define DTOs such as `RepositorySecurityAdvisoriesView`, `RepositorySecurityAdvisoryRow`, `RepositorySecurityAdvisoryDetail`, `RepositorySecurityAdvisoryPackage`, `RepositorySecurityAdvisoryCredit`, `RepositorySecurityAdvisoryCollaborator`, `RepositorySecurityAdvisoryTimelineEvent`, `CvssSummary`, `CweReference`, and `AdvisoryViewer`.
- `crates/api/src/routes/repositories.rs`: register authenticated list/detail routes, enforce repository read permission, hide draft advisories from non-maintainers, keep private repository outsider responses as 404, normalize `state`, `severity`, `q`, `page`, `page_size`, and return standard 422 envelopes for invalid filters.
- Advisory helper functions: generate stable GHSA-style IDs for local drafts, render/sanitize Markdown through the existing Markdown pipeline, validate known severity/state enums, and avoid leaking private draft details through overview counts.
- `crates/api/tests/repository_security_advisories_contract.rs`: seed published/draft advisories, package metadata, CVSS/CWE values, credits/collaborators, permissions, and verify DTO shape, pagination/filter behavior, private/draft privacy, Markdown sanitization, and no session/OAuth/env leakage.
- `web/src/lib/api.ts` and server fetch helpers: add typed advisory list/detail DTOs without adding client-side auth.

**Verification**: focused Rust contract tests against `TEST_DATABASE_URL`, `cd web && npx tsc --noEmit --pretty false`, focused Biome if web types are touched, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Advisories List - filters, pagination, and maintainer entry points

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/security/advisories` inside `RepositorySecurityShell` with Advisories selected. Readers see published advisories only. Maintainers see draft and published tabs, New draft security advisory, filter controls, and pagination backed by Phase 1.

**Key changes**:
- `web/src/app/[owner]/[repo]/security/advisories/page.tsx`: server-fetch repository metadata and advisory list data, preserve URL query params, and render forbidden/unavailable states without leaking private draft counts.
- `web/src/components/RepositorySecurityAdvisoriesPage.tsx`: render heading, description, New draft action, Published/Draft/Withdrawn tabs when authorized, severity filter, search input, sort/page controls, advisory rows with status icon, title, GHSA id, dates, author avatar/login, severity chip, package summary, and empty states using Editorial primitives/tokens only.
- `web/src/components/RepositorySecurityAdvisoryFilters.tsx`: client URL controls for query/severity/state/sort/page with outside-click/Escape handling, accessible selected state, Clear/Apply actions, and no inert controls.
- `web/src/lib/navigation.ts`: add advisory list/detail/new/edit/publish href helpers that safely encode owners, repositories, and GHSA identifiers.
- `web/tests/repository-security-advisories-page.test.tsx`: cover active Security sidebar state, reader vs maintainer row visibility, filters, pagination, concrete row/new links, long title/package wrapping, no `href="#"`, no unsafe HTML, mobile no-overflow, and Editorial banned-value guardrails.
- `web/tests/e2e/repository-security-advisories.spec.ts`: focused signed-session browser smoke for list filters, row navigation, draft privacy, no dead controls, and screenshot `ralph/screenshots/build/code-security-005-phase2-advisories-list.jpg`.

**Verification**: focused Vitest and focused Playwright smoke, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Advisory Detail and Metadata Editing - Markdown, CVSS, CWE, credits

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/security/advisories/{ghsa_id}` plus maintainer edit flow for draft and published advisory metadata. The detail page should render the full advisory with CVSS score/modal data, package ranges, CVE/CWE disclosure, credits, collaborators, timeline, and edit controls. Mutations validate fields server-side and write audit/notification events.

**Key changes**:
- `PATCH /api/repos/{owner}/{repo}/security/advisories/{ghsa_id}`: support updating title, CVE, package ecosystem/name, vulnerable/patched ranges, severity/CVSS vector, CWE IDs, Markdown summary/details, credits, and collaborators with maintainer permission, archived repository guardrails, stale update conflicts, and standard validation envelopes.
- `crates/api/src/domain/repository_security_advisories.rs`: implement validation for GHSA/CVE/CVSS/CWE/package fields, credit type enums, collaborator permissions, sanitized Markdown rendering, audit-event redaction, and notification fanout.
- `web/src/app/[owner]/[repo]/security/advisories/[ghsaId]/page.tsx` and mutation proxy/server action: fetch detail and forward authenticated edits to Rust.
- `web/src/components/RepositorySecurityAdvisoryDetailPage.tsx`: render title, severity/state chips, author, GHSA/CVE IDs, package metadata, affected/patched ranges, Markdown details, CVSS score button/modal, base metrics table, CWE disclosure, credits, collaborator list, timeline, and reader/maintainer states.
- `web/src/components/RepositorySecurityAdvisoryEditor.tsx`: form controls for advisory metadata, Markdown write/preview tabs, CVSS/CWE/package validation feedback, credit/collaborator editors, pending/success/error states, and server-confirmed refresh.
- Extend Rust, Vitest, and Playwright coverage for detail rendering, edit validation, CVSS/CWE parsing, credit/collaborator writes, reader mutation denial, archived repository denial, notification writes, audit redaction, no dead controls, and screenshot `ralph/screenshots/build/code-security-005-phase3-advisory-detail.jpg`.

**Verification**: focused Rust mutation contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 4: Draft Creation and Publishing - real advisory lifecycle

**Done**: [x]

**Scope**: Add `/{owner}/{repo}/security/advisories/new` and publish actions. Maintainers can create draft advisories with required title, add optional metadata, keep drafts private to collaborators, and publish to the public list. Publishing records immutable publish metadata, creates notification/audit rows, and optionally links a dependency advisory feed row for package vulnerabilities.

**Key changes**:
- `POST /api/repos/{owner}/{repo}/security/advisories`: create a draft advisory with required title and optional metadata, generated GHSA id, initial author/collaborator, sanitized Markdown, audit event, and notifications for advisory collaborators.
- `POST /api/repos/{owner}/{repo}/security/advisories/{ghsa_id}/publish`: validate maintainer permission, current draft state, required publishable fields, optional CVE request/existing CVE metadata, patched-version warnings, status transition, public published timestamp, notification fanout, and dependency advisory feed linkage without calling GitHub APIs.
- `web/src/app/[owner]/[repo]/security/advisories/new/page.tsx`: render the draft creation form through `RepositorySecurityShell` with Editorial form controls and no GitHub visual styling.
- `web/src/components/RepositorySecurityAdvisoryPublishPanel.tsx`: publish readiness checklist, CVE/package/CVSS warnings, confirm action, pending/success/error states, and concrete post-publish destination.
- Extend tests for draft creation, required title validation, invalid CVSS/CWE/package fields, draft privacy, publish validation, published row visibility, dependency advisory linkage, notification fanout, audit redaction, and mobile no-overflow.

**Verification**: focused Rust create/publish contracts, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/code-security-005-phase4-create-publish.jpg`, mandatory Editorial banned-value scan, then `make check && make test`; run full `make test-e2e` when the local wrapper is stable.

---

## Phase 5: API Docs, Edge Cases, Browser Evidence, QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `code-security-005` only after list, detail, edit, create, publish, collaboration/credits, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document advisory list/detail/create/edit/publish endpoints with auth/privacy gates, filters, pagination, field validation, draft privacy, publish rules, notifications, audit events, dependency advisory linkage, and no-secret error envelopes.
- Final Rust tests: cover hidden private repositories, draft vs published visibility, malformed filters, stale edit conflicts, duplicate GHSA/CVE handling, invalid CVSS vectors/CWE IDs/package ecosystems, long Markdown/title wrapping, credit/collaborator permission edges, archived repositories, notification updates, audit redaction, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard focus order through filters/forms/modals, CVSS modal and metrics table, detail action controls, publish validation, semantic severity/state chips, long text wrapping, mobile no-overflow, no unsafe HTML, no `href="#"`, no inert click handlers, and Editorial token compliance.
- `web/tests/e2e/repository-security-advisories.spec.ts`: full signed-session browser sweep for list filters, detail page, edit metadata, create draft, publish flow, reader draft privacy, disabled/empty states, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/code-security-005/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `code-security-005.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/code-security-005-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
