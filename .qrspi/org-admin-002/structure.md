# Structure Outline: org-admin-002 Organization Profile Settings

**Ticket**: `org-admin-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-1.jsx`, `design/project/og-shell.jsx`, existing personal profile settings in `web/src/components/PersonalProfileSettingsForm.tsx`, existing repository settings shell in `web/src/components/RepositoryGeneralSettingsPage.tsx`, existing organization profile routes in `web/src/app/orgs/[org]/...`, current organization domain/routes in `crates/api/src/domain/organizations.rs` and `crates/api/src/routes/organizations.rs`, and the organization creation contract from `.qrspi/org-admin-001/structure.md`.
**Date**: 2026-05-04

## Phase 1: Organization Settings API Contract - owner-only profile read/write

**Done**: [ ]

**Scope**: Add the authenticated Rust contract for organization profile settings. Organization owners can load and update public profile fields, contact/billing metadata, social accounts, and location through real Postgres-backed endpoints. Non-owners and anonymous users must not see settings-only data.

**Key changes**:
- `crates/api/migrations/<timestamp>_organization_profile_settings.*.sql`: add only missing additive storage for `organization_social_accounts`, billing/contact email fields if not already present, profile settings audit indexes, and avatar metadata placeholders compatible with the existing S3 avatar contract.
- `crates/api/src/domain/organizations.rs`: add `OrganizationProfileSettings`, `OrganizationProfileSettingsPatch`, `OrganizationSocialAccount`, `OrganizationSettingsViewerState`, and `OrganizationSettingsError` DTOs/helpers for owner permission checks, profile reads, field validation, partial updates, social account replacement, and audit insertion.
- Reuse existing `organizations.name`, `slug`, `description`, `avatar_url`, `website_url`, `location`, `contact_email`, `company_name`, `terms_of_service_type`, `organization_policy_settings`, `organization_memberships`, and `organization_audit_events` instead of duplicating state.
- Validation rules: display name cannot be blank; description/location/URL/social inputs have bounded lengths; URLs must be HTTP(S); public email/billing email must be valid email syntax; social providers are limited to four stable rows; settings responses redact private member or billing-only details from non-owners.
- `crates/api/src/routes/organizations.rs`: add `GET /api/orgs/:org/settings/profile` and `PATCH /api/orgs/:org/settings/profile` with standard `401`, `403`, `404`, `422`, conflict, and database-unavailable envelopes.
- `crates/api/tests/organization_profile_settings_contract.rs`: cover owner read/write, anonymous 401, member/non-owner 403, case-insensitive slug lookup, validation errors, social account persistence/replacement, contact/billing email persistence, audit redaction, and no raw stack traces or secrets in errors.

**Verification**: focused `organization_profile_settings_contract` against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Organization Settings Shell - replace the placeholder profile route

**Done**: [ ]

**Scope**: Replace `/orgs/{org}/settings` placeholder routing with the real protected organization settings shell and profile page at `/organizations/{org}/settings/profile`, with compatibility links from `/orgs/{org}/settings`. The UI must use the Editorial system while matching GitHub's settings information architecture: top organization context, left grouped settings sidebar, context switcher, and right content cards.

**Key changes**:
- `web/src/app/organizations/[org]/settings/profile/page.tsx`: add the canonical protected settings route, fetch signed-in session/shell context and the Phase 1 settings DTO server-side, and render owner-only or forbidden/unavailable states.
- `web/src/app/orgs/[org]/settings/page.tsx`: redirect or link to `/organizations/{org}/settings/profile` so existing org profile settings links remain concrete.
- `web/src/components/OrganizationSettingsShell.tsx`: add an Editorial settings shell with organization avatar/slug context, grouped sidebar links, active state, personal/organization context switcher, and disabled/out-of-scope billing entries instead of inert links.
- `web/src/components/OrganizationProfileSettingsForm.tsx`: render public profile, contact, social, location, billing/contact metadata, and danger-zone sections using `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.t-h1`/`.t-h2`/`.t-label`, and tokenized colors only.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed settings DTOs and server fetch helpers.
- `web/tests/organization-profile-settings-page.test.tsx`: cover shell navigation, context switcher links, initial field rendering, disabled unsupported billing/payment pages, owner/forbidden/unavailable states, no `href="#"`, no unnamed buttons, and Editorial token/primitives expectations.
- `web/tests/e2e/organization-settings-profile.spec.ts`: focused smoke for opening the settings route as an owner, checking sidebar/context switcher behavior, and saving `ralph/screenshots/build/org-admin-002-phase2-settings-shell.jpg`.

**Verification**: focused Vitest and Playwright smoke, mandatory Editorial banned-value scan, then full `make check && make test`; run `make test-e2e` when the local database and auth seed are stable.

---

## Phase 3: Profile, Contact, and Social Mutations - save independent sections

**Done**: [ ]

**Scope**: Wire every editable public-profile/contact/social section to the Rust API. Each Save button must submit a real PATCH, show confirmed success or field-level error feedback, preserve server state on failure, and refresh the page data without local-only persistence.

**Key changes**:
- `web/src/app/organizations/[org]/settings/profile/actions/route.ts`: add same-origin forwarding for `PATCH /api/orgs/{org}/settings/profile` using the current cookie and standard JSON errors.
- `web/src/components/OrganizationProfileSettingsForm.tsx`: add section-specific save handlers for display name/description/URL/location/public email, social accounts, and billing/contact metadata; include pending states, dirty-state affordances, validation summaries, and success/error flash messages.
- Keep avatar upload disabled with a concrete unavailable state unless the existing S3 avatar route can be reused safely for organizations in this phase; no dead upload controls.
- `crates/api/tests/organization_profile_settings_contract.rs`: extend for partial patches, invalid URLs/emails, social account ordering/removal, long values, audit event payloads, and owner-only update guarantees.
- `web/tests/organization-profile-settings-page.test.tsx`: cover section saves, field errors, success messages, failed-save rollback, social row add/remove bounds, disabled avatar state, no local-only state changes, and no dead controls.
- `web/tests/e2e/organization-settings-profile.spec.ts`: update profile fields through the browser, reload to verify persistence, test validation recovery, and save `ralph/screenshots/build/org-admin-002-phase3-profile-save.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright mutation smoke with fresh DB seed, then `make check && make test`; run full `make test-e2e` if the Playwright auth/database setup is healthy.

---

## Phase 4: Rename, Archive, Delete, and UX Guardrails - destructive controls without billing scope

**Done**: [ ]

**Scope**: Complete the danger-zone affordances required by the PRD without adding billing or paid-plan provisioning. Rename validates slug availability and requires confirmation; archive/delete dialogs require typed slug and stay disabled until prerequisites are met. Unsupported destructive backend operations may remain disabled only when the UI clearly states why and has no inert handlers.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: add rename support if safely implementable through the existing slug availability rules, owner checks, conflict handling, and audit rows; expose archive/delete eligibility state even if archive/delete execution remains unsupported for MVP safety.
- `crates/api/src/routes/organizations.rs`: add `PATCH` handling for slug rename or a narrow `POST /api/orgs/:org/settings/profile/rename` route only if it simplifies validation; keep delete/archive unsupported responses explicit and non-destructive unless implemented with retention semantics.
- `web/src/components/OrganizationProfileSettingsForm.tsx`: add rename confirmation, archive dialog, and delete dialog with typed slug matching, disabled prerequisites, focus trapping/return, mobile wrapping, and no `onClick={() => {}}` placeholders.
- `web/tests/organization-profile-settings-page.test.tsx`: cover slug availability feedback, rename confirmation, conflict errors, typed-slug enablement, archive/delete disabled states, keyboard focus recovery, long organization names/emails/social handles, and mobile no-overflow classes.
- `web/tests/e2e/organization-settings-profile.spec.ts`: browser smoke for rename validation/recovery, confirmation dialogs, typed slug enablement, mobile no-overflow, and screenshot `ralph/screenshots/build/org-admin-002-phase4-danger-zone.jpg`.
- Ensure private org/settings details do not leak through reserved/duplicate slug errors.

**Verification**: focused Rust contract, focused Vitest, focused Playwright desktop/mobile smoke, then `make check && make test`; run full `make test-e2e` unless local environment instability is documented.

---

## Phase 5: Docs, Browser QA Handoff, and Build Pass - finish org-admin-002

**Done**: [ ]

**Scope**: Document the organization settings API and complete final build-loop bookkeeping. Mark `org-admin-002.build_pass=true` only after owner-only API access, Editorial settings shell, real save flows, rename/danger-zone guardrails, browser smoke, and QA handoff are verified.

**Key changes**:
- `web/src/lib/api-docs.ts`: document `GET /api/orgs/{org}/settings/profile`, `PATCH /api/orgs/{org}/settings/profile`, and any rename/danger-zone endpoints added in Phase 4, including auth, owner permissions, validation envelopes, social account shape, contact/billing fields, audit behavior, and unsupported billing/deletion constraints.
- `web/tests/api-docs.test.tsx`: assert the organization settings endpoints, request/response shapes, validation notes, audit notes, and no raw stack trace/secret examples are rendered.
- `web/tests/e2e/organization-settings-profile.spec.ts`: final smoke for settings navigation, context switcher, public profile save, contact/social save, validation failures, rename guardrails, danger dialogs, mobile no-overflow, and no dead controls.
- `ralph/screenshots/build/`: save final desktop shell, profile form, validation-error, danger-zone, and mobile screenshots when test data allows.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `org-admin-002.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session DASHBOARD_E2E_SKIP_MIGRATIONS=1 make test-e2e`; browser smoke proves every visible organization settings button, sidebar item, context switcher entry, form, checkbox, dialog, error state, CTA, and redirect has concrete behavior; mandatory Editorial banned-value scan returns zero matches.
