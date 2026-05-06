# Structure Outline: milestones-001 Repository Milestone Management

**Ticket**: `milestones-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-3.jsx`, `design/project/og-screens-4.jsx`, existing issue/PR list and detail implementations, `crates/api/migrations/202604300003_collaboration.up.sql`, `crates/api/src/domain/issues.rs`, `crates/api/src/domain/pulls.rs`, `crates/api/src/routes/repositories.rs`, `web/src/app/[owner]/[repo]/milestones/page.tsx`, `web/src/app/[owner]/[repo]/issues/page.tsx`, `web/src/app/[owner]/[repo]/pulls/page.tsx`, `web/src/components/IssueTimeline.tsx`, and `web/src/lib/api.ts`.
**Date**: 2026-05-06

## Existing Baseline

The collaboration foundation already has repository-scoped `milestones`, `issues.milestone_id`, PRs backed by issues, issue/PR list milestone filters, issue/PR metadata controls, and Editorial repository shells. The current `/{owner}/{repo}/milestones` route is a placeholder. `milestones-001` should turn milestones into a first-class planning surface while reusing existing issue/PR metadata contracts and permission checks. Keep Rust API/session authority, repository-safe 404s, structured error envelopes, rendered Markdown cache for descriptions, and Editorial primitives/tokens. Do not introduce GitHub visual styling, Primer/Octicons, JS-side auth, external GitHub APIs, or fake read-only milestone data.

## Phase 1: Milestone Management API Contract - list/detail and lifecycle mutations work through Rust

**Done**: [ ]

**Scope**: Add the Rust API contract for repository milestone list, detail, create, edit, close, reopen, and delete. Readers can list open/closed milestones and open details; writers can mutate milestone metadata and state. This phase should be API-first and independently testable without replacing the placeholder UI.

**Key changes**:
- Add only additive migration support if needed: `milestone_events`, `milestone_item_order`, milestone query indexes, and audit/activity metadata. Reuse existing `milestones`, `issues.milestone_id`, PR issue backing, and `rendered_markdown_cache` where possible.
- `crates/api/src/domain/milestones.rs`: introduce DTOs such as `RepositoryMilestonesView`, `RepositoryMilestoneSummary`, `RepositoryMilestoneDetail`, `MilestoneProgress`, `MilestoneIssueItem`, `MilestoneMutationRequest`, `MilestoneStateResult`, `MilestoneSort`, and `MilestoneViewer`.
- List behavior: support `state=open|closed|all`, `sort=updated-desc|due-asc|due-desc|complete-asc|complete-desc|alpha-asc|alpha-desc|issues-desc|issues-asc`, bounded pagination, open/closed counts, progress percent, open issue count, closed issue count, and issue/PR count href data.
- Detail behavior: return milestone metadata, rendered description, progress, open/closed item counts, open/closed issue and PR-backed issue rows, selected-item metadata, and `viewer.canEditMilestones`.
- Validation behavior: require repository visibility/read permission for list/detail, write/maintain permission for mutations, non-empty unique title, optional bounded Markdown description, optional due date, valid state transitions, archived-repository denial, and repository-safe private 404s.
- Mutation behavior: create/update/close/reopen/delete `milestones`, clear associated issue/PR milestone foreign keys on delete, record issue/PR timeline metadata changes when associations are cleared, insert milestone events/audit rows, update rendered Markdown cache for descriptions, and never leak env/session secrets.
- `crates/api/src/routes/repositories.rs`: register `GET/POST /api/repos/{owner}/{repo}/milestones`, `GET/PATCH/DELETE /api/repos/{owner}/{repo}/milestones/{milestone_id}`, and `POST /api/repos/{owner}/{repo}/milestones/{milestone_id}/{close|reopen}` or an equivalent REST-shaped state endpoint.
- `web/src/lib/api.ts`: add typed signed-cookie helpers for milestone list/detail/create/update/close/reopen/delete without JS-side auth.
- Tests: focused Rust contract coverage for list state tabs, all sort modes, detail rows/progress, create/edit validation, close/reopen, delete clearing issue/PR associations, permission denial, archived/private behavior, rendered description cache, audit/event rows, and no secret leakage.

**Verification**: focused Rust milestone contract tests against `TEST_DATABASE_URL` when available, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, `cd web && npx tsc --noEmit --pretty false`, focused Biome for touched web types, then full `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Milestones List - milestones are searchable, sortable, and manageable

**Done**: [ ]

**Scope**: Replace the placeholder `/{owner}/{repo}/milestones` route with a real Editorial milestones page backed by Phase 1. Users should switch Open/Closed tabs, sort milestones through URL state, follow count links, and use permissioned create/edit/delete controls with no dead interactions.

**Key changes**:
- `web/src/app/[owner]/[repo]/milestones/page.tsx`: server-fetch repository metadata and milestone list, preserve query state, keep the repository shell active near Issues, and render unavailable/private/error states through existing Editorial components.
- `web/src/components/RepositoryMilestonesPage.tsx`: render `Milestones`, Open/Closed tabs with counts, Sort menu with all PRD options, milestone rows/cards, due-date or `No due date`, last-updated text, progress bar, percent complete, open/closed issue count links, and permissioned New/Edit/Delete controls.
- `web/src/components/RepositoryMilestoneForm.tsx`: shared create/edit form with title, Markdown description, due-date input, preview support if cheap through the Rust Markdown preview path, Save/Cancel states, validation errors, and no inert buttons.
- Navigation helpers: add milestone list/detail/edit/new href helpers plus issue/PR query hrefs preserving `milestone:` qualifiers and `state` filters.
- Sort/state behavior: changing tabs or sort updates shareable URL state without losing other query params; empty open/closed states include working CTAs for writers and useful reader copy.
- Tests: focused Vitest coverage for Open/Closed tabs, sort URL state, progress math rendering, count links, create/edit form payloads, delete confirmation, permissioned controls, long title/description wrapping, no `href="#"`, no dead handlers, and Editorial banned-value guardrails.
- Browser smoke: load `/namuh-eng/opengithub/milestones` against a local API-compatible stub or real dev API, switch Closed tab, change sort, create a milestone, edit it, delete after confirmation, follow open/closed issue count links, check no dead controls or horizontal overflow, and save `ralph/screenshots/build/milestones-001-phase2-list.jpg`.

**Verification**: focused Vitest and browser smoke, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Milestone Detail and Issue/PR Assignment - planning view scopes and mutates work

**Done**: [ ]

**Scope**: Add `/{owner}/{repo}/milestones/{milestone}` detail behavior and connect milestone assignment from issue and pull request sidebars. Users should inspect milestone work, create a prefilled issue, change item metadata, and see milestone progress update through Rust-owned contracts.

**Key changes**:
- `web/src/app/[owner]/[repo]/milestones/[milestone]/page.tsx`: server-fetch detail data, render Back to Milestones, title, state pill, due date, last updated, description, progress, Open/Closed item tabs, selectable item rows, and permissioned Edit/Close/Reopen/Delete controls.
- `web/src/components/RepositoryMilestoneDetailPage.tsx`: render issue and PR-backed issue rows with labels, comments, linked PRs, assignees, selected-count state, and bulk metadata affordances that are enabled only for viewers with permission.
- Issue creation integration: New issue from a milestone should link to the existing issue-create route with milestone preselected, and the backend should validate the milestone belongs to the repository.
- Existing issue/PR metadata routes: ensure milestone changes validate repository ownership, update `issues.milestone_id` transactionally, emit issue/PR timeline events, audit rows, notification hooks where existing metadata changes do so, and return updated milestone summaries.
- Sidebar UI: reuse existing metadata controls with an Editorial milestone picker/search, No milestone option, Save/Cancel states, structured errors, and read-only rendering for unauthorized viewers.
- Tests: focused Rust contract tests for issue/PR milestone add/remove, cross-repository milestone rejection, unauthorized denial, timeline/audit metadata, New issue preselection contract, and progress recomputation; focused Vitest for detail tabs, item selection, count links, picker search/select/save, error preservation, and Editorial guardrails.
- Browser smoke: open a seeded milestone detail, switch Open/Closed tabs, follow New issue with milestone preselected, change milestone on an issue and PR sidebar, verify progress/count changes, check no dead controls or horizontal overflow, and save `ralph/screenshots/build/milestones-001-phase3-detail-assignment.jpg`.

**Verification**: focused Rust metadata/detail tests, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 4: Milestone Prioritization and Lifecycle Edge Cases - ordering and state transitions are durable

**Done**: [ ]

**Scope**: Add drag/drop prioritization for milestones with 500 or fewer open items and harden lifecycle edge cases. Writers should reorder open work items, close/reopen/delete safely, and get deterministic conflict/error behavior.

**Key changes**:
- Rust API: add `PATCH /api/repos/{owner}/{repo}/milestones/{milestone_id}/order` or equivalent to persist ordered issue/PR item IDs in `milestone_item_order`, capped at 500 open items, with repository ownership validation and stale-order conflict detection.
- Detail API: return item order metadata, `canReorder`, and a clear reason when the milestone has more than 500 open items or the viewer lacks permission.
- UI: implement keyboard-accessible up/down controls and drag/drop reorder only where supported by existing dependencies; avoid new dependencies unless explicitly necessary. Persist reorder through the Rust endpoint, show pending/error states, and preserve row focus.
- Lifecycle controls: close/reopen/delete from detail and list should use real Rust mutations, show confirmation where destructive, redirect to the correct list/detail page, and leave associated issues/PRs intact with milestone cleared on delete.
- Edge cases: duplicate titles across case variants, due dates in the past, milestones with no issues, milestones with all closed items, deleted milestones in stale URLs, archived repositories, private repository anonymous access, and very long Markdown descriptions.
- Tests: focused Rust contract tests for reorder success, over-500 denial, cross-repository IDs, stale conflicts, close/reopen/delete side effects, association clearing, and audit rows; focused Vitest for reorder controls, disabled over-limit state, lifecycle button flows, confirmation/cancel, stale error display, mobile no-overflow, and Editorial guardrails.
- Browser smoke: reorder a small milestone, verify persisted order after reload, close and reopen a milestone, delete a milestone and verify associated item milestone chips clear, check no dead controls or horizontal overflow, and save `ralph/screenshots/build/milestones-001-phase4-prioritization.jpg`.

**Verification**: focused Rust reorder/lifecycle tests, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 5: Docs, E2E, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `milestones-001` only after milestone list/detail/mutations, issue/PR assignment, prioritization, docs, screenshots, E2E coverage, QA hints, and final bookkeeping are complete.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document milestone list/detail/create/edit/close/reopen/delete/order endpoints, query params, sorting, progress/count semantics, permission boundaries, validation errors, association clearing on delete, issue/PR metadata milestone updates, audit/timeline side effects, and no-secret error envelopes.
- Final Rust tests: cover full milestone lifecycle, list/detail/sort/progress, duplicate and invalid inputs, permission boundaries, private repositories, issue/PR assignment, deletion clearing associations, prioritization order, over-500 cap, stale conflicts, timeline/audit/notification rows, rendered Markdown cache, and no credential/env leakage.
- Final frontend tests: cover Milestones page reader/writer states, form validation, Markdown preview, sort and tab URL state, detail tabs, count links, New issue preselection, sidebar milestone pickers, reorder controls, lifecycle confirmations, long content wrapping, keyboard traversal, mobile no-overflow, and Editorial token compliance.
- `web/tests/e2e/repository-milestones.spec.ts`: signed-session sweep for milestone list, create, edit, close, reopen, detail open/closed tabs, New issue preselection, issue/PR milestone assignment, reorder for a small milestone, delete clearing associations, final desktop/mobile screenshots, no dead controls, and no horizontal overflow.
- `qa-hints.json`: append targets for concurrent milestone edits/deletes, duplicate title collisions, stale reorder conflicts, 500-item reorder cap, time zone handling for due dates, Markdown sanitization, deletion association clearing, notification fanout, private repository leakage, keyboard-only reorder/picker use, and stale progress after metadata changes.
- `build-progress.txt`, `.qrspi/milestones-001/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `milestones-001.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/milestones-001-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, JSON validation for `prd.json` and `qa-hints.json`, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan:

```bash
rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'
```
