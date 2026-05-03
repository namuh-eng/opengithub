# Structure Outline: Personal Access Tokens and Automation Credentials

**Ticket**: `credentials-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, existing token verification paths in `crates/api/src/domain/tokens.rs`, current `/settings/tokens` developer surface, and target personal-access-token/sudo-mode docs.
**Date**: 2026-05-04

## Phase 1: Token Settings API Foundation - list tokens and create sudo grants

**Done**: [x]

**Scope**: Make the security-sensitive backend contract real enough for a signed-in user to see existing PATs, available resource owners/repositories, permission choices, and whether the current session has sudo-mode elevation. No plaintext token generation yet.

**Key changes**:
- `crates/api/migrations/*_personal_access_token_management.*.sql`: add fields needed by the UI and auth layers, including token `description`, `token_type`, `resource_owner_user_id`, `resource_owner_organization_id`, `repository_access`, optional pending/approved state, revoked reason, and a join table `personal_access_token_repositories`; add `sudo_grants` or session elevation metadata with a bounded expiry.
- `crates/api/src/domain/tokens.rs`: extend token DTOs and validation helpers for list/detail context, prefix-only display, expiration state, classic/fine-grained permission normalization, resource-owner membership, and repository selection visibility.
- `crates/api/src/routes/settings_tokens.rs` or equivalent: add authenticated `GET /api/settings/tokens`, `GET /api/settings/tokens/new`, and `POST /api/settings/sudo` endpoints using the existing Rust session guard.
- `crates/api/tests/personal_access_tokens_contract.rs`: cover anonymous 401, list redaction, context resource-owner filtering, sudo grant creation/expiry, invalid sudo confirmation, and no token hashes/plaintext in responses.

**Verification**: focused Rust contract test; `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`; standard `DB_SSL=false CARGO_INCREMENTAL=0 make test` when practical. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Developer Settings Token List - replace read-only placeholder

**Done**: [x]

**Scope**: Turn `/settings/tokens` into a real signed-in Developer Settings shell: token rows, generate menu, expiration/last-used/status metadata, and delete affordances wired to server data, while retaining the existing command snippets as supporting docs.

**Key changes**:
- `web/src/lib/api.ts`: add typed server helpers for token list/context responses and same-origin forwarding helpers where client mutations need cookies.
- `web/src/components/DeveloperTokensPage.tsx`: render real token rows with type/scope chips, resource owner, repository access summary, last-used/expiration state, empty state, and a Generate new token menu for fine-grained and classic flows using Editorial primitives.
- `web/src/app/settings/tokens/page.tsx`: fetch server-side token data through Rust, preserve unavailable/forbidden states, and keep all links concrete (`/settings/personal-access-tokens/new?type=fine_grained`, classic query variant, `/docs/git`, `/docs/api`).
- `web/tests/developer-tokens-page.test.tsx` and focused Playwright smoke: verify populated/empty/unavailable states, generate menu links, no inert anchors/buttons, mobile no-overflow, and Editorial banned-value compliance.

**Verification**: focused Vitest, web typecheck/Biome, focused Playwright `/settings/tokens` smoke with screenshot `ralph/screenshots/build/credentials-001-phase2-token-list.jpg`, then `make check && make test` as the phase gate.

---

## Phase 3: Fine-Grained Token Creation and One-Time Reveal

**Done**: [ ]

**Scope**: Build `/settings/personal-access-tokens/new` as a working fine-grained token creation flow with sudo interstitial, prefilled query parameters, owner/repository selectors, permission matrix, validation, create API, one-time plaintext reveal, and copy feedback.

**Key changes**:
- `crates/api/src/domain/tokens.rs`: add `create_personal_access_token` that generates an `oghp_` secret, stores only `hash_personal_access_token(secret)` and a collision-resistant prefix, validates expiration/resource owner/repository access/permission matrix, inserts selected repository rows, and writes `security_audit_events`.
- `crates/api/src/routes/settings_tokens.rs`: add `POST /api/settings/tokens` for fine-grained creation, gated by active sudo grant and returning plaintext only in the creation response.
- `web/src/app/settings/personal-access-tokens/new/page.tsx` plus client component: render sudo-mode prompt, query-param prefill (`name`, `description`, `target_name`, `expires_in`, permission params), expiration picker, resource owner selector, repository access selector, selected repositories picker, permission matrix, generate button, reveal panel, and copy action.
- Tests: Rust contract coverage for successful create, validation failures, missing/expired sudo, repository access boundaries, one-time plaintext response, and audit redaction; Vitest coverage for prefill, validation, reveal/copy state, and no dead controls.

**Verification**: focused Rust + Vitest; focused Playwright create-flow smoke that generates a disposable token, verifies one-time reveal and persisted prefix on return to `/settings/tokens`, and saves `credentials-001-phase3-token-create.jpg`.

---

## Phase 4: Classic Tokens, Revocation, and Auth Integration Hardening

**Done**: [ ]

**Scope**: Complete the credential lifecycle by supporting classic broad-scope tokens, revocation/delete confirmation, and consistent PAT use across REST API, Git transport, and package registry auth with `last_used_at` updates.

**Key changes**:
- `crates/api/src/domain/tokens.rs`: add classic-scope validation, revoke helpers, reason/status transitions, repository-scope checks used by Git/package/API callers, and redaction helpers for logs/audit.
- `crates/api/src/routes/settings_tokens.rs`: add classic creation mode to `POST /api/settings/tokens`, `DELETE /api/settings/tokens/{token_id}` for revocation, and stable error envelopes for not found/forbidden/already revoked.
- Existing auth callers (`domain/git_transport.rs`, REST auth extractor or bearer-token path, `domain/packages_registry.rs`): ensure PAT scopes and fine-grained repository selections are enforced, expired/revoked tokens fail immediately, and successful use updates `last_used_at`.
- UI: add classic creation option fields, token row revoke dialog with confirmation text, server-confirmed success/error feedback, and stale row handling after delete.
- Tests: cover REST bearer auth, Git clone/push scope denial/success, package pull/push scope denial/success, revoke invalidation, expired token denial, and `last_used_at` refresh without exposing token material.

**Verification**: focused Rust token/git/package/API contract tests; focused token UI Vitest; focused Playwright revoke smoke with screenshot `credentials-001-phase4-token-revoke.jpg`; then full `make check && make test`.

---

## Phase 5: Docs, Browser QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Lock `credentials-001` as a completed feature after docs, end-to-end evidence, security notes, and final bookkeeping are current.

**Key changes**:
- `web/src/lib/api-docs.ts`, `web/src/app/docs/api/page.tsx`, and `/docs/git` content: document PAT auth, creation/list/revoke endpoints, accepted scopes/permissions, Git over HTTPS usage, package registry usage, one-time reveal behavior, and redaction guarantees.
- `qa-hints.json`: append honest QA focus areas for real Google sudo reauth, leaked-token redaction, long token names/descriptions, repository selector scale, concurrent revoke/use races, package/Git auth parity, and accessibility traversal.
- `build-progress.txt` and `prd.json`: record verification evidence and set only `credentials-001.build_pass=true` once all phases pass; leave `qa_pass=false`.
- Final smoke: verify `/settings/tokens`, `/settings/personal-access-tokens/new`, token creation/reveal, revoke, `/docs/git`, and `/docs/api` with no inert controls and saved desktop/mobile screenshots.

**Verification**: `make check && make test && make test-e2e` when the shared migration state permits; otherwise document the known migration blocker and provide focused Playwright/Rust/Vitest evidence. Run the mandatory Editorial banned-value scan before commit.
