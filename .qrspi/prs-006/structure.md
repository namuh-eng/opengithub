# Structure Outline: Pull Request Merge Methods, Mergeability, Diff/Patch, and API Affordances

**Ticket**: `prs-006`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/og-screens-3.jsx`, `.qrspi/prs-004/structure.md`, `.qrspi/prs-005/structure.md`, current `crates/api/src/domain/pulls.rs`, current `crates/api/src/routes/pulls.rs`, current `web/src/components/RepositoryPullRequestDetailPage.tsx`, current `web/src/components/PullRequestFilesChangedPage.tsx`, and current docs/API surfaces.
**Date**: 2026-05-02

## Phase 1: Merge Settings and Branch Rules - repository policy drives mergeability

**Done**: [x]

**Scope**: Replace hardcoded merge methods and placeholder rule checks with repository-owned merge settings plus narrow branch protection/ruleset inputs. The existing PR detail merge box remains live, but its enabled methods, default method, required review/check blockers, and protected-branch messages now come from real persisted policy.

**Key changes**:
- `crates/api/migrations/*_pull_request_merge_policy.*.sql`: add repository merge settings, branch protection/ruleset requirement tables, and required status-check rows if absent; preserve existing `pull_request_checks_summary`, `pull_request_reviews`, and `pull_requests` contracts.
- `crates/api/src/domain/pulls.rs`: add `RepositoryMergeSettings`, `BranchProtectionSummary`, merge policy lookup helpers, and extend `pull_request_mergeability` to compute allowed methods, default method, required approvals, required status checks, stale/failing checks, draft/closed/merged/no-diff blockers, and human-readable rule messages.
- `crates/api/tests/api_pull_request_detail_contract.rs`: seed settings and branch rules; assert disabled merge methods are not returned, required approvals/checks block with stable codes, bypass-free write permissions remain enforced, and public/private redaction is preserved.
- `web/tests/repository-pull-request-detail.test.tsx`: verify the merge method selector only shows enabled methods and renders rule/check/review blockers accessibly with Editorial chips and no inert controls.

**Verification**: focused PR detail contract tests, focused detail Vitest, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, and same-env `make test`. Browser smoke is optional in this backend-policy phase unless the merge box rendering changes substantially.

---

## Phase 2: Atomic Merge Execution - merge commit, squash, and rebase mutate refs safely

**Done**: [x]

**Scope**: Upgrade `POST /api/repos/{owner}/{repo}/pulls/{number}/merge` from a state transition into an atomic merge operation. Authorized users can merge with merge commit, squash, or rebase when repository policy permits; the operation validates the latest mergeability, writes commit/ref metadata, marks the PR merged, closes linked issues, records timeline/audit/activity events, and optionally deletes the head branch when requested.

**Key changes**:
- `crates/api/src/domain/pulls.rs`: add `MergePullRequestInput { method, commit_title, commit_body, delete_branch }`, `merge_pull_request`, method-specific commit synthesis helpers, optimistic base-ref update checks, linked-issue closure extraction, notification/audit/search side effects, and structured `409 merge_blocked` errors with blocker arrays.
- `crates/api/src/routes/pulls.rs`: extend `MergePullRequestRequest` with commit title/body and delete-branch fields; return the updated `PullRequestDetailView` on success and standard blocked envelopes on stale or denied merges.
- `web/src/lib/api.ts` and `web/src/app/[owner]/[repo]/pull/[number]/merge/route.ts`: type and forward the richer merge payload.
- `crates/api/tests/api_pull_request_detail_contract.rs` or `api_pull_request_merge_contract.rs`: cover each merge method, disabled-method rejection, stale blockers, linked issue closure, head branch deletion rules, duplicate merge idempotency, notifications, audit/activity events, and ref/commit persistence.
- `.scratch/prs-006-merge-contract-scenario.sh`: exercise a real Postgres-backed merge through the API with at least one successful method and one blocked method.

**Verification**: focused merge contract scenario, focused Rust merge tests, `make check`, and `make test`. Browser smoke remains optional until the confirmation UI lands in Phase 3.

---

## Phase 3: Editorial Merge Confirmation UI - commit fields, method menu, and branch delete are live

**Done**: [x]

**Scope**: Expand the PR detail merge box into the full Editorial merge workflow. Users can select an allowed method, edit commit title/body, see branch protection/check/review status, choose delete-head-branch when allowed, submit merge, and receive success or structured blocked feedback without dead buttons.

**Key changes**:
- `web/src/components/RepositoryPullRequestDetailPage.tsx`: replace the one-click merge action with an inline confirmation panel/modal using `.btn`, `.chip`, `.card`, `.input`, `.tabs`, `.t-*`, and live tokens only; include commit title/body fields, method-specific helper copy, delete branch checkbox, blocker summary, success state, and recovery actions.
- `web/src/lib/api.ts`: expose mergeability fields needed by the UI, including method availability, delete-branch eligibility, blocker severity, and default commit message if Phase 2 adds them.
- `web/tests/repository-pull-request-detail.test.tsx`: cover method switching, commit field validation, delete-branch payloads, blocked envelopes, merged success state, keyboard/focus behavior, and no `href="#"` or inert controls.
- `web/tests/e2e/repository-pull-request-detail.spec.ts`: signed-session browser smoke for ready merge, blocked merge, method switching, delete branch option, reload persistence, and screenshot `ralph/screenshots/build/prs-006-phase3-merge-confirmation.jpg`.

**Verification**: focused detail Vitest, focused PR detail Playwright smoke, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, and focused same-env `make test-e2e`.

---

## Phase 4: Plaintext Diff/Patch and API Docs - developer affordances match clients

**Done**: [ ]

**Scope**: Add developer-facing raw `.diff` and `.patch` views plus docs for list/create/review/merge endpoints. The raw outputs are permission-aware, text/plain-compatible, bounded, and link from the Files changed and PR detail surfaces without replacing the Editorial UI.

**Key changes**:
- `crates/api/src/routes/pulls.rs`: add route handling for `/api/repos/:owner/:repo/pulls/:number.diff` and `.patch` or equivalent raw route names supported by Axum; preserve normal JSON routes.
- `crates/api/src/domain/pulls.rs`: add `pull_request_plain_diff_for_viewer` and `pull_request_patch_for_viewer` helpers sourced from `pull_request_files`, hunk lines, commits, and refs; return text with stable headers, file hunks, and commit metadata while enforcing public/private permissions.
- `web/src/app/[owner]/[repo]/pull/[number]/diff/route.ts` and `/patch/route.ts` or direct links: proxy raw text with the correct content type and signed cookies.
- `web/src/components/PullRequestFilesChangedPage.tsx` and/or `RepositoryPullRequestDetailPage.tsx`: add concrete `.diff`, `.patch`, and API docs links using Editorial ghost buttons or chips.
- `web/src/lib/api-docs.ts` and `/docs` surface: document pull request list/create/files/review/merge/diff/patch endpoints, request/response examples, errors, and merge blocker codes.
- Tests: Rust raw diff/patch contract for public/private reads and content shape; Vitest docs/link coverage; Playwright smoke for raw links and docs navigation.

**Verification**: focused raw diff/patch Rust tests, focused docs/UI Vitest, focused Playwright raw-link smoke, `make check`, `make test`, and same-env `make test-e2e`.

---

## Phase 5: Final Merge Guardrails and QA Handoff - complete prs-006 safely

**Done**: [ ]

**Scope**: Harden the full feature before marking `prs-006` complete. Validate conflict/no-diff/stale-ref/check/review/ruleset behavior, all merge methods, raw diff/patch privacy, mobile layout, no dead controls, visual compliance, and QA hints.

**Key changes**:
- `crates/api/tests/api_pull_request_detail_contract.rs` and/or `api_pull_request_merge_contract.rs`: final matrix for permissions, private denial/redaction, draft/closed/merged states, disabled methods, required reviews, required checks, branch rules, stale refs, conflict/no-diff blockers, successful merge methods, issue closure, notifications, audit/activity events, branch deletion, and standard error envelopes.
- `web/tests/repository-pull-request-detail.test.tsx` and `web/tests/e2e/repository-pull-request-detail.spec.ts`: final accessibility/dead-control sweep, merge confirmation sweep, raw diff/patch links, docs links, mobile no-overflow, and screenshots `ralph/screenshots/build/prs-006-phase5-final-desktop.jpg` and `ralph/screenshots/build/prs-006-phase5-final-mobile.jpg`.
- `qa-hints.json`: append honest QA notes for real Git object limitations, conflict simulation, raw output bounds, and branch protection edge cases.
- `prd.json`: set only `prs-006.build_pass` to `true` after all verification passes; never touch `qa_pass`.
- `build-progress.txt`: append feature summary, verification evidence, files changed, and known risks.

**Verification**: `.scratch/prs-006-merge-contract-scenario.sh`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`, and mandatory Editorial banned-value scan using the local compatible `rg -n -e` form.
