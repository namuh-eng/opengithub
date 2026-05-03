# Structure Outline: settings-005 Actions Secrets and Variables

**Ticket**: `settings-005`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-settings.jsx` settings shell, existing repository settings navigation in `web/src/lib/navigation.ts`, current secrets placeholder in `web/src/app/[owner]/[repo]/settings/secrets/page.tsx`, existing Actions API/domain in `crates/api/src/routes/actions.rs` and `crates/api/src/domain/actions.rs`, Actions migrations `202605020031` through `202605020036`, repository settings audit patterns from `settings-001` through `settings-004`, and GitHub Actions references in `target-docs/content/rest/actions/secrets.md`, `target-docs/content/rest/actions/variables.md`, `target-docs/content/actions/concepts/security/secrets.md`, and `target-docs/content/actions/concepts/workflows-and-actions/variables.md`.
**Date**: 2026-05-03

## Phase 1: Secrets and Variables API Contract - metadata, encryption, validation, and audit

**Done**: [x]

**Scope**: Add the Rust/Postgres contract for repository Actions secrets and variables under `/api/repos/{owner}/{repo}/settings/secrets`. Repository admins can list metadata, create/update/delete repository-scoped secrets and variables, and see inherited organization/environment metadata when available. Secret values are write-only and never returned by API, audit rows, logs, or test snapshots.

**Key changes**:
- `crates/api/migrations/`: add `actions_secrets`, `actions_variables`, and optional scope/link tables for organization and environment inherited rows if they are not already present. Include repository id, scope kind, scope id/name, normalized name, encrypted value reference or ciphertext envelope, value metadata hash/fingerprint, updated actor/time, visibility policy, and unique constraints per scope/name.
- Define a secret storage envelope suitable for the current AWS contract: prefer server-side envelope encryption with a runtime `ACTIONS_SECRETS_KEY`/KMS-compatible abstraction and a `storage_kind` field that can later move encrypted payloads to AWS Secrets Manager or S3 without changing API shape. Local tests may use deterministic in-process encryption helpers, but plaintext must never be persisted.
- Add DTOs for `RepositoryActionsSecretsSettings`, `ActionsSecretSummary`, `ActionsVariableSummary`, `InheritedActionsSecretSummary`, `InheritedActionsVariableSummary`, `ActionsSecretMutation`, `ActionsVariableMutation`, `ActionsSecretResolutionPolicy`, and structured validation errors.
- `GET /api/repos/{owner}/{repo}/settings/secrets`: repository admins receive editable repository-scoped secret/variable metadata plus inherited org/environment metadata; non-admin readers receive a forbidden response without names for private-only inherited values; anonymous/private repository behavior follows existing repository privacy rules.
- Mutation endpoints: create/update/delete repository secrets and variables. Validate identifier-like names, reserved/default environment variable names, duplicate names, value size bounds, blank secret values on create, blank variable values when disallowed, invalid scope, archived repositories, missing resources, and non-admin writes.
- API responses for secrets must include only name, scope, updated time, updated actor, selected repository/environment visibility metadata, and `hasValue`/`secretConfigured` style booleans. They must not include plaintext, ciphertext, encrypted refs, hashes, or envelope details.
- Successful writes insert `repository_settings_audit_events` with event types like `repository.actions_secret.create`, `repository.actions_secret.update`, `repository.actions_secret.delete`, `repository.actions_variable.create`, and `repository.actions_variable.delete`; audit metadata must redact values and encrypted refs.
- Add `crates/api/tests/repository_actions_secrets_contract.rs` covering admin-only access, secret create/update/delete, variable create/update/delete, uniqueness, name normalization, reserved-name rejection, write-only secret responses, audit redaction, inherited metadata visibility, archived/private repository guardrails, and standard error envelopes without stack/env leakage.

**Verification**: focused `repository_actions_secrets_contract` against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && DB_SSL=false make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Secrets and Variables Shell - tabs, empty states, and inherited metadata

**Done**: [x]

**Scope**: Replace the `/[owner]/[repo]/settings/secrets` placeholder with a real Editorial settings surface backed by the Phase 1 API. The page must cover repository secrets, repository variables, inherited metadata, forbidden/unavailable states, and zero-data empty states without leaking secret values or regressing the locked Editorial visual system.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed DTOs and cookie-backed fetch helpers for the Actions secrets settings read contract, preserving forbidden/unavailable result states.
- `web/src/app/[owner]/[repo]/settings/secrets/page.tsx`: fetch the settings contract server-side and render concrete content inside `RepositorySettingsShell`.
- Add `web/src/components/RepositoryActionsSecretsPage.tsx` with tabs for Secrets and Variables, count chips, empty state with working Add secret/Add variable CTAs, metadata rows, updated timestamps/actors, scope chips, inherited rows, and clear write-only explanatory helper text.
- Secret rows show name, scope, updated time, and Update/Delete controls only; values, encrypted refs, hashes, and ciphertext are never rendered. Variable rows may show values only when the API marks the viewer as permitted.
- Non-admin/forbidden states must not leak private secret names or inherited metadata from organizations/environments the viewer cannot administer. Unavailable states must offer a concrete retry/navigation path.
- Use only Editorial primitives and tokens: `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.t-label`, `.t-mono-sm`, `var(--ink-*)`, `var(--line)`, `var(--accent)`, and semantic chips. Do not introduce GitHub colors, Octicons, Primer imports, nested cards, or dead `href="#"` links.
- Add focused `web/tests/repository-actions-secrets-page.test.tsx` coverage for empty states, secret rows without values, variable rows with permitted values, inherited metadata, forbidden/unavailable states, no inert anchors/buttons, and Editorial primitive/token usage.

**Verification**: focused Vitest for the page, mandatory Editorial banned-value scan, then `make check && make test`. Save a browser screenshot if the E2E seed already exposes a repository with Actions secret fixtures.

---

## Phase 3: Add, Update, and Delete Mutations - confirmed forms and write-only handling

**Done**: [x]

**Scope**: Wire every visible secret and variable action to real server-confirmed writes. Admins must be able to add, update, and delete repository-scoped secrets and variables from the UI with inline success/error feedback, while invalid inputs never update local UI optimistically.

**Key changes**:
- Add same-origin Next.js route handlers or server actions under the secrets settings route that forward authenticated mutations to the Rust API without adding JS-side auth.
- Implement Add/Update secret form with Name input, password-style Secret textarea/input, helper text explaining write-only storage, Add/Update buttons, loading state, and inline validation from both client checks and API envelopes.
- Secret updates require a new value; the UI must never prefill or display the old value. If the API supports metadata-only updates later, keep that as an explicit separate action rather than treating blank secret as "keep existing".
- Implement Add/Update variable form with Name input, Value textarea/input, visibility/scope metadata where supported, displayed value for permitted viewers, and validation feedback.
- Implement delete confirmations for secrets and variables requiring the resource name or a clear typed confirmation. Deletes must refresh from server-confirmed state.
- Extend Rust contract tests for create-vs-update semantics, delete audit rows, duplicate name conflicts, reserved/default variable names, encrypted-value replacement, and deletion of missing resources.
- Extend Vitest coverage for forms, validation errors, write-only secret semantics, delete confirmations, disabled non-admin actions, no local-only state changes, and no placeholder click handlers.
- Add `web/tests/e2e/repository-settings-secrets.spec.ts`: seed an admin repository, create a secret, verify metadata-only row after reload, update the secret without value disclosure, create/update a variable and verify displayed value, delete disposable rows, verify forbidden non-admin state, and save `ralph/screenshots/build/settings-005-phase3-secrets-mutations.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` when local database/dev servers are stable.

---

## Phase 4: Workflow Resolution Integration - secret availability, masking, and fork/event rules

**Done**: [x]

**Scope**: Make Actions secrets and variables operational for workflow execution planning. Workflow runs should resolve the correct repository/organization/environment variables and secrets according to event, fork, reusable-workflow, and environment access rules, while logs and API responses mask any resolved secret values.

**Key changes**:
- Add a focused Rust domain module for Actions runtime context resolution used by workflow dispatch/push trigger creation and future runner/job execution. It should produce separate `variables` and `secrets` maps plus redaction metadata without exposing plaintext outside the bounded execution context.
- Enforce policy rules from the PRD and docs: repository secrets are unavailable to untrusted fork pull request events, inherited organization/environment secrets obey visibility rules, reusable workflow calls receive only explicitly allowed secrets when modeled, and environment secrets are only available after environment requirements pass.
- Integrate variable values into workflow run context metadata where safe; secret values should never be stored in `workflow_runs.event_payload`, job logs, annotations, audit rows, API DTOs, or frontend state.
- Add log masking helpers used by workflow job log creation/download paths so exact secret values and common transformed forms are replaced with a redaction marker before persistence or response.
- Record non-sensitive resolution diagnostics such as counts by scope, blocked reason categories, and policy decisions for QA/debugging.
- Extend Rust tests across `actions.rs` and the new domain module for trusted push/dispatch secret resolution, fork pull-request blocking, inherited metadata policy, environment-gated secrets, variable precedence, masking in log lines/download archives, and no accidental serialization of secret material.
- Add frontend assertions that the settings page and Actions run detail show only secret counts/policy chips or blocked-reason summaries, never values.

**Verification**: focused Actions runtime/secret-resolution tests, focused repository secrets contract, focused Actions log tests, then `make check && make test`. Run focused Playwright for settings plus Actions run detail when seeded data is available.

---

## Phase 5: Guardrails, API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [x]

**Scope**: Finish `settings-005` as a complete vertical slice only after API, UI, mutation flows, workflow resolution, docs, screenshots, and QA handoff are verified. This phase should not add unrelated repository settings areas such as Pages, protected tags, security analysis, or personal access tokens.

**Key changes**:
- Finalize API docs in `web/src/lib/api-docs.ts` for Actions secrets and variables list/create/update/delete endpoints, including auth/visibility, write-only secret behavior, encryption/storage notes, validation errors, audit behavior, inherited metadata, and workflow availability policy.
- Extend `qa-hints.json` with deeper QA targets: real AWS/KMS or Secrets Manager backing if enabled, ciphertext/plaintext leakage scans, fork PR secret blocking, environment-gated secret release, inherited organization scope visibility, long name/value wrapping, concurrent admin edits, audit redaction, and log masking bypass attempts.
- Ensure every visible button/link/form has concrete behavior or an accessible disabled state; verify keyboard navigation through tabs, Add/Update forms, delete confirmations, and empty-state CTAs.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/settings-005-final-secrets-*.jpg` for empty, populated, mutation, variable, inherited, and forbidden states.
- Run the mandatory Editorial banned-value scan before commit and fix any touched-file regressions.
- Update `build-progress.txt`, `.qrspi/settings-005/structure.md`, and `prd.json`; set `settings-005.build_pass=true` only after all implementation phases are complete and verified; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when available, browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
