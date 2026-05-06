# Structure Outline: labels-001 Repository Label Management

**Ticket**: `labels-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-3.jsx`, `design/project/og-screens-4.jsx`, existing issue/PR/discussion list and detail implementations, `crates/api/migrations/202604300003_collaboration.up.sql`, `crates/api/migrations/202605050014_repository_discussions.up.sql`, `crates/api/src/domain/issues.rs`, `crates/api/src/domain/pulls.rs`, `crates/api/src/domain/discussions.rs`, `crates/api/src/routes/repositories.rs`, `crates/api/src/routes/pulls.rs`, `web/src/app/[owner]/[repo]/labels/page.tsx`, `web/src/app/[owner]/[repo]/issues/page.tsx`, `web/src/app/[owner]/[repo]/pulls/page.tsx`, `web/src/components/IssueTimeline.tsx`, and `web/src/lib/api.ts`.
**Date**: 2026-05-06

## Existing Baseline

The collaboration foundation already has repository-scoped `labels`, `issue_labels`, `pull_requests` backed by issues, `discussion_labels`, issue/PR/discussion list filters, issue/PR/discussion metadata proxy routes, and Editorial repository shells. The current `/{owner}/{repo}/labels` page is a placeholder. `labels-001` should turn labels into a first-class management surface and reuse the existing link tables for applying labels to issues, pull requests, and discussions. Keep Rust API/session authority, repository permission checks, structured error envelopes, and Editorial primitives/tokens. Do not introduce GitHub visual styling, Primer/Octicons, JS-side auth, external GitHub APIs, or fake read-only label data.

## Phase 1: Label Management API Contract - list/search/sort and CRUD work through Rust

**Done**: [x]

**Scope**: Add the Rust API contract for repository label management. Readers can list/search/sort labels and follow count links; writers can create, update, and delete labels with audit/timeline-safe side effects. This phase should be API-first and independently testable without replacing the placeholder UI.

**Key changes**:
- Add only additive migration support if needed: label revision/audit metadata such as `label_revisions` or `repository_label_events`, plus indexes for repository/name/count queries. Reuse existing `labels`, `issue_labels`, `discussion_labels`, and PR issue backing where possible.
- `crates/api/src/domain/labels.rs`: introduce DTOs such as `RepositoryLabelsView`, `RepositoryLabelSummary`, `RepositoryLabelCounts`, `RepositoryLabelMutationRequest`, `RepositoryLabelMutationResult`, `RepositoryLabelSort`, and `LabelViewer`.
- Count behavior: compute open issue and open pull request counts from `issue_labels` and the PR issue relationship; compute discussion count for deletion/application side effects even if not shown in the primary row.
- Validation behavior: require repository visibility/read permission for list, write/maintain permission for create/edit/delete, non-empty unique name, normalized six-character hex color without `#`, optional bounded description, and repository-safe 404s for unauthorized private repositories.
- Mutation behavior: create/update/delete `labels`, record label revision/event/audit rows, remove join rows on delete through existing cascades, and insert timeline/activity events where the affected issue/PR/discussion flows already expose label changes.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/labels`, `POST /api/repos/{owner}/{repo}/labels`, `PATCH /api/repos/{owner}/{repo}/labels/{label_id}`, and `DELETE /api/repos/{owner}/{repo}/labels/{label_id}` with bounded query params for `q`, `sort`, `direction`, `page`, and `pageSize`.
- `web/src/lib/api.ts`: add typed fetch helpers for list/create/update/delete with signed-cookie forwarding and no JS auth library.
- Tests: focused Rust contract coverage for list/search/sort/counts, create, edit, delete cascade behavior, duplicate names, invalid colors, permission denial, private repository hiding, structured errors, audit/revision rows, and no secret leakage.

**Verification**: focused Rust label contract tests against `TEST_DATABASE_URL` when available, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, `cd web && npx tsc --noEmit --pretty false`, focused Biome for touched web types, then full `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Labels Page - repository labels are searchable and sortable

**Done**: [x]

**Scope**: Replace the placeholder `/{owner}/{repo}/labels` route with a real reader/writer page backed by Phase 1. Users should search labels, change sort order, follow issue/PR count links, and see permissioned management controls with no dead interactions.

**Key changes**:
- `web/src/app/[owner]/[repo]/labels/page.tsx`: server-fetch repository metadata and labels, keep the repository shell active near Issues, preserve query state, and render unavailable/private/error states through existing Editorial unavailable components.
- `web/src/components/RepositoryLabelsPage.tsx`: render the `Labels` heading, `Search all labels` input, visible count text, Sort radio menu (`Name`, `Total issue count`, `Ascending`, `Descending`) with keyboard hints, compact label rows, color swatches/pills, descriptions, issue/PR count links, and permissioned Edit/Delete/New label controls.
- `web/src/components/RepositoryLabelForm.tsx`: add shared create/edit form with name, description, color input, random-color button, live preview chip, Save/Cancel states, validation errors, and no inert buttons.
- Navigation helpers: add label-list href helpers plus issue/PR query hrefs preserving `label:` qualifiers and `is:open` filters.
- Sort/search behavior: update URL query state without losing the other query params; search filters by name/description and updates the visible count; sort choices reorder rows from the API or a same-shape client state after server load.
- Tests: focused Vitest coverage for search filtering, count text, sort radio behavior, concrete count links, create/edit form payloads, random color generation format, delete confirmation, permissioned control visibility, no `href="#"`, no dead handlers, long label names wrapping, and Editorial banned-value guardrails.
- Browser smoke: load `/namuh-eng/opengithub/labels` against a local API-compatible stub or real dev API, search labels, change sort, open New label, save a label, edit it, delete it after confirmation, follow issue/PR count links, check no dead controls or horizontal overflow, and save `ralph/screenshots/build/labels-001-phase2-labels-page.jpg`.

**Verification**: focused Vitest and browser smoke, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Label Application From Issue and PR Sidebars - metadata controls mutate labels

**Done**: [x]

**Scope**: Wire label apply/dismiss behavior into existing issue and pull request metadata sidebars. Triage-capable users should open a label picker, search/select labels, save changes through Rust, and see timeline/notification side effects reflected in the detail views.

**Key changes**:
- Rust domain: expose reusable label option/apply functions for issues and pull requests, preserving the existing metadata update routes and `labelIds` request shape where already present.
- `crates/api/src/domain/issues.rs` and `crates/api/src/domain/pulls.rs`: ensure metadata updates validate label IDs belong to the repository, compute added/removed labels, update `issue_labels` and PR issue label rows transactionally, emit issue/PR timeline events, audit records, and notification fanout hooks where existing metadata changes do so.
- Existing Next proxy routes `web/src/app/[owner]/[repo]/issues/[number]/metadata/route.ts` and `web/src/app/[owner]/[repo]/pull/[number]/metadata/route.ts`: keep them as thin cookie-forwarding proxies and extend tests around label payloads/status handling if needed.
- `web/src/components/IssueTimeline.tsx` and PR detail components: add or reuse a `LabelPicker` sidebar control with search combobox, color swatches, descriptions, selected chips, Save/Cancel buttons, pending/error states, and read-only rendering for users without triage permission.
- Detail refetch/update behavior: after save, labels update in the sidebar and timeline without a full fake reload; errors preserve selected state and display structured messages.
- Tests: focused Rust contract tests for issue/PR label add/remove, cross-repository label rejection, unauthorized denial, timeline event metadata, notification/audit hooks, and no secret leakage; focused Vitest for picker search, selection/dismiss, save payload, error preservation, read-only state, keyboard traversal, and Editorial guardrails.
- Browser smoke: open a seeded issue and PR detail, change label selection in each sidebar, verify chips and timeline/event feedback, check no dead controls or horizontal overflow, and save `ralph/screenshots/build/labels-001-phase3-issue-pr-application.jpg`.

**Verification**: focused Rust metadata tests, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 4: Discussion Labels and Cross-Surface Consistency - discussions use the same label system

**Done**: [x]

**Scope**: Extend the same label application behavior to discussion sidebars and verify label deletion/editing remains consistent across issues, pull requests, and discussions.

**Key changes**:
- `crates/api/src/domain/discussions.rs`: wire discussion label updates through existing `discussion_labels`, validate repository label ownership, compute added/removed labels, emit discussion events/audit rows, and preserve discussion list/category label filtering behavior.
- Existing Next proxy route `web/src/app/[owner]/[repo]/discussions/[number]/metadata/route.ts`: keep it as a thin proxy and cover label status/error handling.
- Discussion detail/list UI: add or reuse the same Editorial `LabelPicker` for discussions, show labels consistently in list rows and detail sidebars, and ensure label-filter links land on discussion list queries with the selected label.
- Cross-surface consistency: editing a label name/color/description updates issue, PR, and discussion visible chips; deleting a label removes it from all surfaces without deleting conversations.
- Tests: focused Rust discussion label apply/remove, delete cascade across all three join tables, edit propagation, list filter behavior, unauthorized denial, audit/event metadata, and no secret leakage; focused Vitest for discussion label picker, list chip updates, delete cascade UI state, and Editorial guardrails.
- Browser smoke: open a seeded discussion detail, apply and dismiss a label, verify list filter links, then delete a label from the Labels page and verify it disappears from issue/PR/discussion displays; save `ralph/screenshots/build/labels-001-phase4-discussions-consistency.jpg`.

**Verification**: focused Rust discussion/cascade tests, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 5: Docs, E2E, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `labels-001` only after label CRUD, search/sort, issue/PR/discussion application, docs, screenshots, E2E coverage, QA hints, and final bookkeeping are complete.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document repository label list/create/edit/delete, query params, sorting, count semantics, permission boundaries, validation errors, issue/PR/discussion metadata label updates, audit/timeline side effects, and no-secret error envelopes.
- Final Rust tests: cover full label lifecycle, search/sort/counts, duplicate and invalid inputs, permission boundaries, private repositories, deletion cascades, issue/PR/discussion apply/remove side effects, timeline/audit/notification rows, and no credential/env leakage.
- Final frontend tests: cover Labels page reader/writer states, form validation, random color, search/sort URL state, concrete issue/PR/discussion query links, picker behavior across sidebars, delete confirmation, long content wrapping, keyboard traversal, mobile no-overflow, and Editorial token compliance.
- `web/tests/e2e/repository-labels.spec.ts`: signed-session sweep for label list, create, edit, delete, issue label apply/dismiss, PR label apply/dismiss, discussion label apply/dismiss when seeded, final desktop/mobile screenshots, no dead controls, and no horizontal overflow.
- `qa-hints.json`: append targets for concurrent label edits/deletes, case-insensitive name collisions, color contrast of arbitrary user labels, long labels/descriptions, deletion cascade across all join tables, notification fanout, private repository leakage, keyboard-only label picker use, and stale UI after metadata changes.
- `build-progress.txt`, `.qrspi/labels-001/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `labels-001.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/labels-001-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, JSON validation for `prd.json` and `qa-hints.json`, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan:

```bash
rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'
```
