# Structure Outline: Personal Public Profile Settings

**Ticket**: `personal-settings-001`
**Page**: `/settings/profile`
**Date**: 2026-05-03

## Phase 1: Settings data contract and persistence

**Done**: [x]

- Additive migration for profile settings columns on `users`, email choices, four social account slots, S3-modeled avatar rows, and `security_audit_events`.
- Authenticated Rust API reads and writes `/api/user/settings/profile` and `/api/user/settings/profile/avatar`.
- Saves display identity, optional clearable fields, time/localization preferences, privacy toggles, social accounts, avatar upload/remove state, and profile/audit side effects.

## Phase 2: Editorial settings UI

**Done**: [x]

- Replace the profile placeholder with signed-in `SettingsShell` content titled `Public profile`.
- Add working profile, avatar, social account, contribution/activity, and profile-settings controls using Editorial primitives and CSS variables.
- Inline validation, dirty-state Update profile button, avatar preview/reset/remove, and short success flash all work without page navigation.

## Phase 3: Focused tests and guardrails

**Done**: [x]

- Add Rust API contract tests for profile read/write, optional clears, privacy side effects, avatar validation, avatar remove, and audit rows.
- Add Vitest coverage for required fields, dirty save behavior, inline URL/avatar errors, privacy save, no inert links, and named buttons.
- Run focused tests, web typecheck, mandatory Editorial banned-value scan, `make check`, and `CARGO_INCREMENTAL=0 make test` before marking the PRD build pass.

## Phase 4: Bookkeeping and QA handoff

**Done**: [x]

- Update only `personal-settings-001` in `prd.json` when checks pass.
- Append concise build evidence to `build-progress.txt` and `qa-hints.json`.
- Commit and push `feat/opengithub-personal-settings-001`; do not merge.
