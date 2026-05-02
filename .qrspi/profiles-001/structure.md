# Structure Outline: profiles-001 Public User Profile Overview

**Ticket**: `profiles-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-profiles.jsx`, current `web/src/app/[owner]/page.tsx`, current `web/src/components/ProfileOrgShell.tsx`, current `crates/api/src/routes/users.rs`, current `crates/api/src/domain/identity.rs`, existing repository/social migrations, and `target-docs/auth-flow.md`.
**Date**: 2026-05-02

## Phase 1: Profile Overview API Contract - public profile data, visibility boundaries, and pinned repository rows

**Done**: [x]

**Scope**: Replace the placeholder profile route with a real Rust read contract for `/{user}`. The API should resolve usernames case-insensitively, return the profile identity card, README/body metadata, pinned repositories, achievements, organization memberships, contribution summary, tab counts, and viewer state. Private profiles must keep only the allowed public identity fields visible and suppress follower counts, secondary tabs, achievements, activity, and contribution graph.

**Key changes**:
- Add an additive SQLx migration for profile-owned tables and columns that are missing from the foundation: public profile fields on `users`, `user_profile_readmes`, `profile_pins`, `profile_contribution_days`, `profile_contribution_events`, `achievements`, `user_achievements`, `user_blocks`, and `user_reports`. Reuse existing `user_follows`, `repository_stars`, `repository_forks`, `repositories`, `repository_languages`, and `organization_memberships` tables instead of duplicating social data.
- `crates/api/src/domain/profiles.rs`: introduce `PublicUserProfile`, `ProfileIdentity`, `ProfilePinnedRepository`, `ProfileContributionDay`, `ProfileAchievement`, `ProfileViewerState`, and helpers for username lookup, visibility filtering, follower/following counts, pinned repository permission checks, language/count summaries, and private-profile redaction.
- `crates/api/src/routes/users.rs`: add `GET /api/users/{username}/profile` with standard error envelopes, optional signed-session viewer detection, 404 for missing users, and no private repository leakage.
- `crates/api/src/domain/mod.rs`, `crates/api/src/routes/mod.rs`, and `crates/api/src/lib.rs`: wire the profile domain/route without changing `/api/user`.
- `crates/api/tests/profile_overview_contract.rs`: seed public and private users, follows, organizations, public/private repositories, languages, pins, contribution days, achievements, and verify response shape, tab counts, private-profile redaction, missing-user errors, and anonymous/signed-in viewer states.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed `PublicUserProfile` DTOs and server fetch helper for profile pages.

**Verification**: focused `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false cargo test --test profile_overview_contract -- --nocapture`, then same-env `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Profile Shell - `/{user}` renders the public overview from real data

**Done**: [x]

**Scope**: Replace the current placeholder `ProfileOrgShell` usage for user profiles with a full Editorial profile overview: left identity column, tab bar, README/bio area, pinned repository grid, achievement badges, organization memberships, and contribution graph summary. Visuals must follow the Editorial system, not GitHub Primer chrome: use `web/src/app/og.css` tokens, `.btn`, `.chip`, `.card`, `.tabs`, `.av`, `.t-*` classes, and the profile layout language from `design/project/wf-profiles.jsx`.

**Key changes**:
- `web/src/app/[owner]/page.tsx`: load the profile API, distinguish user profile data from organization skeleton destinations where needed, handle 404/unavailable states, preserve `?tab=` navigation, and keep anonymous public access.
- `web/src/components/UserProfilePage.tsx`: render the profile overview with stable responsive dimensions, accessible tab navigation, identity metadata, follower/following counts when visible, README/bio card, up to six pinned repository cards, achievements, organization chips, and contribution summary.
- `web/src/components/ProfileOrgShell.tsx`: either specialize or keep as the organization skeleton; avoid forcing the richer user profile into the old single-card placeholder.
- `web/src/lib/navigation.ts`: ensure `PROFILE_TABS`, `activeProfileTab`, and `profileTabHref` keep tab URLs stable without losing the identity column.
- `web/tests/user-profile-page.test.tsx`: assert public profile rendering, private-profile redaction, pinned repository cards/links, tab hrefs, no dead `href="#"`, no placeholder text, and Editorial token/primitives usage.
- `web/tests/e2e/profile-overview.spec.ts`: add a public browser smoke that opens a seeded user profile, follows tab links, checks pinned repository navigation, and saves `ralph/screenshots/build/profiles-001-phase2-overview.jpg`.

**Verification**: focused Vitest for the profile page, focused Playwright smoke for `/{user}`, mandatory Editorial banned-value scan, then `make check`, `make test`, and `make test-e2e` if the seeded profile scenario is stable.

---

## Phase 3: Follow, Block, and Report Controls - visible profile actions write real records

**Done**: [x]

**Scope**: Make every profile action concrete. The Follow button must toggle `user_follows` for signed-in viewers with optimistic UI and rollback. The block/report menu must open a real menu; signed-out users get a login-gated modal, and signed-in users can create block/report records with validation and feedback. The controls must be hidden or disabled for private profiles and self-profile states where the action is not meaningful.

**Key changes**:
- `crates/api/src/domain/profiles.rs`: add idempotent `follow_user`, `unfollow_user`, `block_user`, and `report_user` helpers with self-action guards, follower count reconciliation, feed-event creation where appropriate, and audit events for block/report actions.
- `crates/api/src/routes/users.rs`: add `PUT /api/users/{username}/follow`, `DELETE /api/users/{username}/follow`, `PUT /api/users/{username}/block`, and `POST /api/users/{username}/reports`; all require authentication and return either updated profile viewer/count state or a standard error envelope.
- `web/src/app/[owner]/profile-actions.ts` or same-origin route handlers: forward signed cookies to Rust for profile mutations and normalize error envelopes for client components.
- `web/src/components/UserProfileActions.tsx`: implement Follow, More menu, block dialog, report dialog, loading states, optimistic follower count updates, rollback on failure, signed-out login modal, and self-profile behavior.
- `crates/api/tests/profile_social_actions.rs`, `web/tests/user-profile-actions.test.tsx`, and Playwright coverage: cover anonymous 401/login gate, self-action 422, idempotent follow/unfollow, block/report persistence, optimistic rollback, private profile suppression, and no leaked stack traces.

**Verification**: focused Rust social-action contract, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/profiles-001-phase3-actions.jpg`, then `make check`, `make test`, and `make test-e2e`.

---

## Phase 4: Contribution Graph and Accessibility - heatmap cells, tooltips, year selector, and responsive behavior

**Done**: [ ]

**Scope**: Complete the contribution graph and profile overview ergonomics. Contribution cells must expose date/count labels to screen readers, hover/focus tooltips, month labels, intensity legend, annual total, and a year selector that updates URL/data. Private profiles must hide the graph entirely. Mobile layouts must avoid horizontal overflow while keeping the identity column and main content scannable.

**Key changes**:
- `crates/api/src/domain/profiles.rs`: add year-bounded contribution queries, intensity bucketing, annual totals, and event-count summaries with date clamping and viewer-safe privacy behavior.
- `crates/api/src/routes/users.rs`: support `?year=` on the profile read endpoint or add `GET /api/users/{username}/profile/contributions?year=YYYY` with the same visibility rules.
- `web/src/components/ProfileContributionGraph.tsx`: render a keyboard-accessible grid/heatmap with month labels, weekday context, legend, tooltip text, year selector, and screen-reader summaries without hardcoded banned colors.
- `web/src/components/UserProfilePage.tsx`: integrate the graph, empty states, private-profile state, responsive breakpoint behavior, and tab-specific page titles.
- `web/tests/user-profile-contributions.test.tsx` and `web/tests/e2e/profile-overview.spec.ts`: cover screen-reader labels, tooltip/focus behavior, year navigation, private-profile suppression, mobile no-overflow, long username/repository truncation, and empty contribution states.

**Verification**: focused Rust contribution contract, focused Vitest accessibility assertions, focused desktop/mobile Playwright smoke saving `ralph/screenshots/build/profiles-001-phase4-contributions.jpg`, then `make check`, `make test`, and `make test-e2e`.

---

## Phase 5: Profile Guardrails and QA Handoff - finish profiles-001

**Done**: [ ]

**Scope**: Harden privacy, accessibility, API contracts, visual consistency, and bookkeeping. Mark `profiles-001.build_pass=true` only after the public profile overview uses real API data, pinned repositories navigate to real repository pages, actions persist, private-profile redaction is proven, and no visible controls are dead.

**Key changes**:
- `crates/api/tests/profile_overview_contract.rs` and `crates/api/tests/profile_social_actions.rs`: final coverage for username casing, missing users, private profile redaction, repository permission boundaries, organization membership visibility, follower/following counts, block/report validation, contribution year limits, and absence of stack traces/secrets in error responses.
- `web/tests/user-profile-page.test.tsx`, `web/tests/user-profile-actions.test.tsx`, and `web/tests/user-profile-contributions.test.tsx`: final coverage for tabs, pinned cards, achievements, empty states, private state, action feedback, keyboard navigation, focus management, mobile text fitting, and no dead controls.
- `web/tests/e2e/profile-overview.spec.ts`: signed-in and anonymous desktop/mobile smoke for overview, tab navigation, pinned repo links, follow/unfollow, block/report login gate, contribution tooltip/year selector, private profile redaction, and screenshots.
- `ralph/screenshots/build/`: save final public profile desktop, public profile mobile, follow/menu state, contribution tooltip, and private profile screenshots.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `profiles-001.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`; browser smoke proves every profile button, menu, dialog, tab, pinned-card row, year selector, tooltip, and empty-state CTA has a concrete action; mandatory Editorial banned-value scan returns zero matches.
