# Structure Outline: settings-003 Branch Protection and Rulesets

**Ticket**: `settings-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-settings.jsx` (`Branches_A`, `Branches_B`), existing repository settings shell in `web/src/components/RepositorySettingsShell.tsx`, current branches placeholder in `web/src/app/[owner]/[repo]/settings/branches/page.tsx`, existing mergeability branch-protection read path in `crates/api/src/domain/pulls.rs`, existing Git push sync path in `crates/api/src/domain/git_transport.rs`, existing branch rule schema in `crates/api/migrations/202605020030_pull_request_merge_policy.up.sql`, access/admin patterns from `settings-001` and `settings-002`, and audit-event persistence in `repository_settings_audit_events`.
**Date**: 2026-05-03

## Phase 1: Branch Policy API Contract - rules, rulesets, refs, and admin visibility

**Done**: [ ]

**Scope**: Add the Rust/Postgres read/write contract for repository branch policy settings under `/api/repos/{owner}/{repo}/settings/branches`. This phase should preserve the existing `repository_branch_protection_rules` and `repository_required_status_checks` behavior used by PR mergeability, while extending it into a complete settings contract with rulesets, default-branch context, matching refs, viewer capability metadata, and audit events.

**Key changes**:
- `crates/api/migrations/`: add additive schema for missing fields on `repository_branch_protection_rules` and a new `repository_rulesets` / rule-condition shape if the current minimal table cannot model enforcement state, bypass actors, signed commits, linear history, merge queue, deployments, locked branch, push restrictions, force pushes, deletions, and evaluate/disabled rulesets.
- Keep the existing minimal mergeability columns compatible: `pattern`, `required_approving_review_count`, `requires_up_to_date_branch`, and `repository_required_status_checks.context` must continue to feed `crates/api/src/domain/pulls.rs`.
- Add DTOs for `RepositoryBranchSettings`, `RepositoryBranchRule`, `RepositoryRuleset`, `BranchPolicyRequirement`, `BypassActor`, `BranchPolicyEvaluation`, `BranchPolicyMutation`, `DefaultBranchSummary`, and structured validation errors.
- `GET /api/repos/{owner}/{repo}/settings/branches`: allow repository admins to see editable rules and non-admin readers to see active/evaluate policy explanations without mutation capability; anonymous/private access must follow existing repository privacy behavior.
- Include branch refs from `repository_git_refs`, default branch metadata, matching branch counts, required status-check context suggestions from `check_runs`/`check_suites`, and viewer state (`canEdit`, disabled reasons).
- Add mutation endpoints for create/update/delete branch protection rules and rulesets. Validate blank/invalid patterns, duplicate exact patterns, unsupported enforcement values, negative review counts, missing status check contexts, invalid bypass actors, and unsafe default-branch deletion allowances.
- Every successful write inserts `repository_settings_audit_events` with changed fields and before/after JSON.
- Add `crates/api/tests/repository_branch_settings_contract.rs` covering admin read/write/delete, reader-visible active policy explanations, private repo privacy, default branch summary, duplicate/conflicting patterns, fnmatch-style pattern matching, status-check suggestions, bypass actor validation, audit events, and redacted structured errors.

**Verification**: focused `repository_branch_settings_contract` against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && DB_SSL=false make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Branches Settings Shell - default branch card, rules list, and read-only explanations

**Done**: [ ]

**Scope**: Replace the `/[owner]/[repo]/settings/branches` placeholder with a real Editorial Branches settings page backed by the Phase 1 API. The page must show the default branch card, branch protection rules, rulesets, matching branch counts, and non-admin read-only policy explanations without GitHub/Primer visual regression.

**Key changes**:
- `web/src/lib/api.ts` and server-session helpers: add typed branch-settings DTOs and cookie-backed fetch helpers preserving forbidden/unavailable states.
- `web/src/app/[owner]/[repo]/settings/branches/page.tsx`: fetch the branch settings contract server-side and render the concrete page inside `RepositorySettingsShell`.
- Add `web/src/components/RepositoryBranchSettingsPage.tsx` with default branch summary, rule/ruleset tabs, rule cards, ruleset cards, matching branch chips, requirement chips, bypass actor summaries, active/evaluate/disabled state chips, empty states, and working New branch protection rule / New ruleset CTAs.
- Non-admin readers may see active/evaluate policy cards and blocked-push/blocked-merge explanations, but all mutation controls must be absent or disabled with accessible reasons.
- Use only Editorial primitives and tokens: `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.t-label`, `.t-mono-sm`, `var(--ink-*)`, `var(--line)`, `var(--accent)`, and semantic chips. Do not introduce GitHub colors, Octicons, Primer imports, nested cards, or dead `href="#"` links.
- Add focused `web/tests/repository-branch-settings-page.test.tsx` coverage for default branch rendering, active/evaluate/disabled chips, read-only non-admin state, empty states with concrete CTAs, no inert anchors/buttons, and Editorial primitive/token usage.

**Verification**: focused Vitest for the page, mandatory Editorial banned-value scan, then `make check && make test`. Save a browser screenshot if the E2E seed already exposes a repository with branch refs.

---

## Phase 3: Branch Rule and Ruleset Mutations - editors, confirmations, and server-confirmed state

**Done**: [ ]

**Scope**: Wire every visible branch-policy action to real API writes. Admins must be able to create, edit, disable/evaluate/activate, and delete branch protection rules/rulesets through confirmed server state; invalid or conflicting policies must show inline errors and never update the UI optimistically.

**Key changes**:
- Add same-origin Next.js route handlers or server actions under the branches settings route that forward authenticated mutation requests to the Rust API without adding JS-side auth.
- Implement Branch protection rule editor: pattern input, matching preview, enforcement status segmented control, required reviews, required status checks, conversation resolution, signed commits, linear history, merge queue, deployment requirements, locked branch, restrict pushes, allow force pushes, and allow deletions.
- Implement Ruleset editor with pattern/target conditions, active/evaluate/disabled state, bypass actor list, and the same requirement groups where the API supports them.
- Add add/remove controls for status-check contexts and bypass actors with accessible validation, loading states, success/error feedback, and server-confirmed refresh.
- Add delete/disable confirmations for rules and rulesets; default-branch destructive allowances must require an explicit confirmation label or be rejected by API validation.
- Extend Rust contract tests for conflict detection, update/delete audit events, evaluate-only rules, disabled rules, bypass actor persistence, and required status-check normalization.
- Extend Vitest coverage for editors, validation errors, status-check/bypass actor controls, delete confirmations, disabled non-admin actions, and no local-only state changes.
- Add `web/tests/e2e/repository-settings-branches.spec.ts`: seed an admin repository, create a rule for `main`, edit required checks/reviews, create an evaluate-only ruleset, delete or disable a disposable rule, reload to verify persistence, and save `ralph/screenshots/build/settings-003-phase3-branches-mutations.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` when local database/dev servers are stable.

---

## Phase 4: Policy Evaluation Integration - PR mergeability and Git push enforcement

**Done**: [ ]

**Scope**: Make branch protection and rulesets operational. Pull request mergeability and Git push handling must aggregate all applicable branch policies and choose the most restrictive requirement. Push attempts that violate active policies must fail with structured errors, while evaluate-only rules record evaluations without blocking.

**Key changes**:
- Extract shared branch-policy evaluation into a focused Rust domain module used by `crates/api/src/domain/pulls.rs`, `crates/api/src/domain/git_transport.rs`, and the settings API. Avoid duplicating pattern matching and requirement aggregation.
- Replace the simple first-match `branch_protection_summary` behavior with most-restrictive aggregation across matching branch rules and active rulesets: highest required review count, union of required status checks, any required conversation resolution, signed commits, linear history, merge queue/deployments, locked branch, push restrictions, force-push/deletion restrictions, and bypass actor allowances.
- Upgrade pattern matching from the current single-asterisk helper to bounded fnmatch-style matching used consistently by UI previews, PR mergeability, and push enforcement.
- Integrate with existing PR mergeability response shape while preserving compatibility for existing tests: return protected pattern/ruleset summaries, required checks, required reviews, and human-readable blocked reasons.
- Enforce active push rules in Git smart HTTP push flow before accepting/syncing refs where feasible. Block restricted pushes, locked branches, force pushes, deletions, and missing bypass permissions with redacted structured errors; evaluate-only rules must write `repository_rule_evaluations` rows without blocking.
- Add/extend Rust tests across PR detail and Git transport for most-restrictive aggregation, bypass actors, evaluate-only logging, deletion/force-push blocking, private repo privacy, and backward compatibility with existing minimal branch-rule fixtures.
- Add frontend assertions that PR detail and Branches settings show the same active requirements and blocked reasons for a seeded protected branch.

**Verification**: focused `api_pull_request_detail_contract`, focused git transport branch-policy tests, focused branch settings tests, then `make check && make test`. Run focused Playwright for PR detail plus branches settings when seeded data is available.

---

## Phase 5: Guardrails, Documentation, Browser Smoke, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `settings-003` as a complete vertical slice only after API, UI, mutation flows, PR mergeability, Git push enforcement, docs, screenshots, and QA handoff are verified. This phase should not add unrelated settings areas such as tags, webhooks, Pages, secrets, or security logs.

**Key changes**:
- Finalize API docs in `web/src/lib/api-docs.ts` for branch settings read/write endpoints, policy evaluation behavior, rulesets, bypass actors, status-check requirements, audit events, and push/merge enforcement responses.
- Extend `qa-hints.json` with deeper QA targets: concurrent admin edits, CODEOWNERS-specific review semantics if still deferred, real Git force-push/deletion attempts through smart HTTP, ruleset evaluate-only reporting, bypass actor permission changes, long pattern wrapping, and private repository leakage checks.
- Ensure every visible button/link/form has concrete behavior or an accessible disabled state; verify keyboard navigation through editors and confirmations.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/settings-003-final-branches-*.jpg` for admin and non-admin/read-only states.
- Run the mandatory Editorial banned-value scan before commit and fix any touched-file regressions.
- Update `build-progress.txt`, `.qrspi/settings-003/structure.md`, and `prd.json`; set `settings-003.build_pass=true` only after all implementation phases are complete and verified; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when available, browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
