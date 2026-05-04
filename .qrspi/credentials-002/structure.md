# Structure Outline: SSH, GPG, and Vigilant Signing Keys

**Ticket**: `credentials-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, existing `/settings/keys` placeholder, `credentials-001` token settings patterns, and current commit/tag signature metadata surfaces.
**Date**: 2026-05-04

## Phase 1: Key Settings API Foundation

**Done**: [x]

**Scope**: Add durable storage and authenticated API contracts for SSH keys, GPG keys, and the user's vigilant-mode preference. This phase should make the backend authoritative before replacing the placeholder UI.

**Key changes**:
- `crates/api/migrations/*_personal_signing_keys.*.sql`: add `ssh_keys`, `gpg_keys`, `gpg_key_emails` or equivalent normalized metadata, `users.vigilant_mode`, last-used/revoked timestamps, unique per-user fingerprints, and audit-event indexes. Keep revoked keys for history instead of deleting rows.
- `crates/api/src/domain/signing_keys.rs`: add DTOs and validation helpers for SSH public key parsing, SHA256 fingerprint derivation, allowed SSH key kinds, armored GPG public key parsing, GPG fingerprint/email extraction, duplicate detection, and vigilant-mode updates.
- `crates/api/src/routes/settings_keys.rs`: add authenticated `GET /api/settings/keys`, `POST /api/settings/keys/ssh`, `DELETE /api/settings/keys/ssh/{key_id}`, `POST /api/settings/keys/gpg`, `DELETE /api/settings/keys/gpg/{key_id}`, and `PATCH /api/settings/keys/vigilant-mode`.
- `crates/api/src/main.rs`: register the settings-key route group without weakening existing token settings auth.
- `crates/api/tests/personal_signing_keys_contract.rs`: cover anonymous 401, empty settings payload, SSH create validation, fingerprint uniqueness, revoke retention/audit events, GPG parse validation, vigilant-mode persistence, and redaction of raw key material where only fingerprints should appear.

**Verification**: focused Rust contract test; `cargo fmt --all --check`; `cargo check -p opengithub-api`; mandatory Editorial banned-value scan even though UI is not touched. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial SSH Key Management UI

**Done**: [x]

**Scope**: Replace `/settings/keys` placeholder with a real signed-in settings page for SSH authentication/signing keys using the Editorial design system.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed key-settings fetch/mutation helpers and same-origin forwarding for cookie-backed POST/DELETE/PATCH actions.
- `web/src/app/settings/keys/page.tsx`: fetch server-side key settings and render unavailable/unauthorized states explicitly.
- `web/src/components/DeveloperKeysPage.tsx`: render the settings shell, SSH key rows with type, title, SHA256 fingerprint, added date/source, last-used state, read/write access, and Delete controls; include a New SSH key form with title, key type, public key textarea, validation feedback, save/cancel, and server-confirmed append.
- `web/src/app/settings/keys/actions/route.ts`: forward SSH create/delete mutations to Rust with session cookies and stable error envelopes.
- `web/tests/developer-keys-page.test.tsx`: cover populated/empty SSH states, create validation, successful append, delete confirmation, failed delete rollback, no inert links/buttons, and Editorial class/token usage.
- `web/tests/e2e/developer-keys.spec.ts`: smoke signed-in `/settings/keys`, create a disposable SSH key, delete it, assert the row state and save `ralph/screenshots/build/credentials-002-phase2-ssh-keys.jpg`.

**Verification**: focused Vitest and Playwright smoke; `cd web && npx tsc --noEmit --pretty false`; focused Biome check; `make check && make test` when practical.

---

## Phase 3: GPG Keys and Vigilant Mode UI

**Done**: [x]

**Scope**: Complete the visible settings surface by adding GPG key management and vigilant-mode preference updates.

**Key changes**:
- Extend `DeveloperKeysPage` with a GPG keys section: New GPG key button/form, armored public key textarea, parsed fingerprint/email summary after save, empty state, row delete confirmation, and audit-preserving revoke semantics.
- Add the Vigilant mode card with checkbox labeled "Flag unsigned commits as unverified", explanatory copy, immediate save/error feedback, and disabled state while saving.
- Extend `web/src/app/settings/keys/actions/route.ts` for GPG create/delete and vigilant-mode PATCH actions.
- Extend `web/tests/developer-keys-page.test.tsx` for GPG validation/success/delete, vigilant-mode toggle persistence, disabled-in-flight state, and no dead controls.
- Extend `web/tests/e2e/developer-keys.spec.ts` to cover adding a fixture GPG key, toggling vigilant mode, mobile no-overflow, and screenshot `ralph/screenshots/build/credentials-002-phase3-gpg-vigilant.jpg`.

**Verification**: focused Rust validation tests if parser behavior changes; focused Vitest; focused Playwright; web typecheck/Biome; mandatory Editorial banned-value scan.

---

## Phase 4: Commit and Git Authentication Integration

**Done**: [x]

**Scope**: Connect stored keys to product behavior: SSH keys should be usable for future Git auth checks, and signing metadata should influence commit/tag verification presentation for commits attributed to the user.

**Key changes**:
- `crates/api/src/domain/git_transport.rs` or a new SSH auth domain boundary: add key lookup helpers that can validate an incoming SSH public-key fingerprint once SSH transport is introduced, and update `last_used_at` without exposing key material.
- Commit/tag signature presentation domains (`crates/api/src/domain/releases.rs`, commit history/detail modules when present): annotate user-attributed commits/tags as verified, unverified, or vigilant-unverified based on matching GPG fingerprints and `users.vigilant_mode`.
- Frontend commit/release surfaces that already show signature metadata: render vigilant-mode unverified messaging with `.chip.warn`/Editorial tokens, preserving existing release tag signature summaries.
- Rust tests: cover fingerprint match/mismatch, revoked-key exclusion, last-used update helper, vigilant-mode unsigned commit classification, and non-leakage for private key material.
- Web tests: cover visible verification copy and semantic chip state without introducing GitHub colors.

**Verification**: focused Rust tests for signing classification and SSH lookup helper; focused web tests for commit/release presentation; `make check && make test`. Browser smoke is optional unless a visible commit/release page changes in this phase.

---

## Phase 5: API Docs, Browser QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Lock `credentials-002` as a completed feature after docs, browser evidence, QA hints, and build-loop bookkeeping are current.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document key settings endpoints, SSH/GPG validation rules, duplicate fingerprint behavior, revoke-retention semantics, vigilant-mode preference, and error envelopes.
- `/docs/git`: add SSH public-key and commit-signing guidance while preserving PAT guidance from `credentials-001`.
- `qa-hints.json`: append QA focus areas for real SSH transport parity, malformed SSH/GPG parser edge cases, duplicate-key races, revoked-key audit retention, vigilant-mode commit presentation, accessibility traversal, and mobile density.
- `build-progress.txt` and `prd.json`: record final verification and set only `credentials-002.build_pass=true`; leave `qa_pass=false`.
- Final browser smoke: verify `/settings/keys` SSH create/delete, GPG create/delete, vigilant-mode toggle, `/docs/git`, `/docs/api`, no inert controls, and save desktop/mobile screenshots.

**Verification**: mandatory Editorial banned-value scan; `make check`; `make test`; `make test-e2e` when the shared migration state permits, otherwise document the blocker and include focused Rust/Vitest/Playwright evidence with a fresh migrated database.
