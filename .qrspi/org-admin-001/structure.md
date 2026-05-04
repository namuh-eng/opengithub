# Structure Outline: org-admin-001 Organization Creation Flow

**Ticket**: `org-admin-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-1.jsx`, `design/project/og-shell.jsx`, existing organization profile/repository outlines in `.qrspi/orgs-001/structure.md` and `.qrspi/orgs-002/structure.md`, current organization routes/domain in `crates/api/src/routes/organizations.rs` and `crates/api/src/domain/organizations.rs`, existing repository create owner patterns in `crates/api/src/domain/repositories.rs`, and the current protected placeholder at `web/src/app/organizations/new/page.tsx`.
**Date**: 2026-05-04

## Phase 1: Organization Creation API Contract - create a Free organization with owner membership

**Done**: [x]

**Scope**: Add the authenticated Rust contract for organization creation while keeping the UI placeholder intact. A signed-in user can create an organization with normalized slug, display name, contact email, ownership type, optional company name, terms acceptance, default policy rows, owner membership, and audit event. Paid plan provisioning stays explicitly out of scope.

**Key changes**:
- `crates/api/migrations/<timestamp>_organization_create_flow.*.sql`: add missing additive columns/tables for `organizations.contact_email`, `terms_of_service_type`, `company_name`, optional `ownership_type`, `organization_policy_settings`, `organization_audit_events`, and `reserved_slugs`; preserve existing org/profile reads.
- `crates/api/src/domain/organizations.rs`: add `CreateOrganizationRequest`, `OrganizationSlugAvailability`, `CreatedOrganization`, `OrganizationCreateError`, `normalize_organization_slug`, `validate_organization_slug`, `organization_slug_availability`, and `create_organization_from_signup`.
- `crates/api/src/routes/organizations.rs`: add authenticated `GET /api/organizations/slug-availability?name=...` and `POST /api/organizations`, mapping validation, duplicate, reserved-slug, rate-limit, unauthenticated, and database errors to the standard envelope.
- `crates/api/src/domain/repositories.rs`: leave the older internal `create_organization` helper alone unless it can safely delegate to the new org-domain function without broad repository-create churn.
- `crates/api/tests/organization_create_contract.rs`: cover anonymous 401, slug normalization, reserved words, duplicate slugs, invalid contact email, missing terms, personal/business ownership validation, default policy row creation, owner membership, audit redaction, and response redirect target.

**Verification**: focused `organization_create_contract`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Plan Picker and Setup Form - render the protected flow with live slug validation

**Done**: [x]

**Scope**: Replace `/organizations/new` placeholder with the real Editorial two-step flow. The first step shows Free, Team, and Enterprise plan cards; Free opens the setup form, while paid plans are disabled/info-only. The setup form derives a slug preview as the user types and validates availability through the Phase 1 endpoint without submitting.

**Key changes**:
- `web/src/app/organizations/new/page.tsx`: fetch signed-in session/shell context and render a real organization creation page instead of `PlaceholderPage`.
- `web/src/components/OrganizationCreatePage.tsx`: add the plan picker and setup form using `.card`, `.btn`, `.chip`, `.input`, `.t-h1`/`.t-h2`/`.t-label`, Editorial spacing, and no GitHub colors or Primer imports.
- `web/src/lib/api.ts`: add typed `getOrganizationSlugAvailabilityFromCookie` helper and create-flow DTOs.
- `web/src/app/organizations/new/actions.ts` or same-origin route handler: forward slug availability checks to Rust with the current cookie so the browser never calls a fake API.
- `web/tests/organization-create-page.test.tsx`: cover plan-card state, Free CTA opening the form, disabled paid plans, slug preview normalization, availability/error callouts, conditional company field, required terms checkbox, no dead buttons, no `href="#"`, and Editorial token usage.
- `web/tests/e2e/organization-create.spec.ts`: focused smoke for opening `/organizations/new`, selecting Free, typing names, seeing the normalized URL preview, reserved-name feedback, and desktop screenshot `ralph/screenshots/build/org-admin-001-phase2-plan-picker.jpg`.

**Verification**: focused Vitest and Playwright smoke, then full `make check && make test`; run `make test-e2e` when the local database and auth seed are stable.

---

## Phase 3: Submit Create and Redirect - the form creates the organization end to end

**Done**: [x]

**Scope**: Wire the setup form submission to `POST /api/organizations` and redirect to the new organization profile or settings profile route after a successful create. Client and server validation must agree, inline failures must stay on the form, and double-submit/race behavior must be deterministic.

**Key changes**:
- `web/src/lib/api.ts`: add `createOrganizationFromCookie` with typed success/error handling and no secret/header leakage.
- `web/src/app/organizations/new/actions.ts`: add a server action or route-backed submit handler that validates required fields, calls Rust, returns field-level errors, and redirects to `/{org}` or `/organizations/{org}/settings/profile` according to the Phase 1 response.
- `web/src/components/OrganizationCreatePage.tsx`: add submit pending state, inline error summary, field errors for duplicate/reserved slug and invalid email, success redirect behavior, disabled duplicate submission, and concrete Cancel/Back controls.
- `crates/api/tests/organization_create_contract.rs`: extend for concurrent duplicate create attempts, canonical slug casing, contact email persistence, company-name persistence only for business/institution ownership, and audit payload redaction.
- `web/tests/organization-create-page.test.tsx`: cover successful action payload, inline failures, double-submit disabled state, redirect target, and keeping user-entered values after validation errors.
- `web/tests/e2e/organization-create.spec.ts`: create a uniquely named organization through the browser, assert navigation lands on the real org route, and verify the org appears in app-shell organization navigation after refresh.

**Verification**: focused Rust contract, focused Vitest, focused Playwright create smoke with fresh DB seed, then `make check && make test`; run full `make test-e2e` if the Playwright auth/database setup is healthy.

---

## Phase 4: Organization Defaults and Creation UX Guardrails - policy settings, rate limits, and accessibility

**Done**: [x]

**Scope**: Harden the flow for product-grade edge cases: default organization policy settings, audit trails, rate-limit feedback, reserved-slug source data, keyboard/accessibility behavior, mobile layout, and no-overflow form states. This phase should not add billing or paid-plan provisioning.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: finalize default policy settings for new orgs, validate reserved-slug seed data, enforce bounded create attempts per actor/IP if an existing rate-limit primitive is available, and ensure error messages do not leak private users/orgs.
- `crates/api/src/routes/organizations.rs`: expose consistent `429` rate-limit envelopes when limits trigger; keep `/api/health` behavior unchanged.
- `web/src/components/OrganizationCreatePage.tsx`: add mobile-responsive plan/form layouts, accessible radio groups/checkboxes/error associations, keyboard focus management between plan picker and form, and stable dimensions for long org names/emails.
- `web/tests/organization-create-page.test.tsx`: extend for ARIA labels/descriptions, keyboard traversal, long-word wrapping, rate-limit error rendering, and mobile no-overflow expectations.
- `web/tests/e2e/organization-create.spec.ts`: add mobile smoke, keyboard-only Free-plan selection, terms checkbox, error recovery, and screenshot `ralph/screenshots/build/org-admin-001-phase4-mobile.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright desktop/mobile smoke, then `make check && make test`; run full `make test-e2e` unless local environment instability is documented.

---

## Phase 5: Docs, Browser QA Handoff, and Build Pass - finish org-admin-001

**Done**: [ ]

**Scope**: Document the organization creation API and complete final build-loop bookkeeping. Mark `org-admin-001.build_pass=true` only after API contract, Editorial UI, real browser create flow, validation, redirect behavior, and QA handoff are verified.

**Key changes**:
- `web/src/lib/api-docs.ts`: document `GET /api/organizations/slug-availability` and `POST /api/organizations`, including auth, validation envelopes, reserved slugs, terms acceptance, default owner membership, default policy settings, and audit behavior.
- `web/tests/api-docs.test.tsx`: assert the organization creation endpoints, request/response shapes, validation notes, and no raw stack trace/secret examples are rendered.
- `web/tests/e2e/organization-create.spec.ts`: final smoke for plan picker, live slug validation, create submit, duplicate/reserved-name failures, redirect to new org, app-shell org presence, mobile no-overflow, and no dead controls.
- `ralph/screenshots/build/`: save final desktop plan picker, setup form, success/org-redirect, validation-error, and mobile screenshots when test data allows.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `org-admin-001.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session DASHBOARD_E2E_SKIP_MIGRATIONS=1 make test-e2e`; browser smoke proves every visible organization creation button, card, input, checkbox, radio, error state, CTA, and redirect has concrete behavior; mandatory Editorial banned-value scan returns zero matches.
