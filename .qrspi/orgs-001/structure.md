# Structure Outline: orgs-001 Organization Overview Profile

**Ticket**: `orgs-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-profiles.jsx`, existing profile QRSPI outlines, current `crates/api/src/domain/profiles.rs`, current organization placeholder route, and current profile UI patterns.
**Date**: 2026-05-02

## Phase 1: Organization Overview API Contract - identity, visibility, counts, and preview data

**Done**: [x]

**Scope**: Add the Rust read contract for `GET /api/orgs/{org}/profile` so the organization overview can render from real Postgres data. This phase should expose organization identity, verified domains, public/member-aware repository previews, pinned repositories, public people preview, topic/language summaries, sponsorship placeholder state, and tab counts. The contract must hide private/internal repository and member details from anonymous or non-member viewers while allowing members/admins to see permitted internal data.

**Key changes**:
- `crates/api/migrations/<timestamp>_organization_profiles.*.sql`: add only the missing additive metadata for organization profile rendering, such as verified domains, public member visibility flags if absent, organization pins if `profile_pins` cannot represent org-owned pins, and indexes for organization-owned repository/profile reads. Reuse existing `organizations`, `organization_memberships`, `repositories`, `repository_languages`, `repository_topics`, stars/forks/issues/PR counts, packages, and projects placeholders where possible.
- `crates/api/src/domain/organizations.rs` or a narrowly named profile module: add `PublicOrganizationProfile`, organization identity DTOs, verified-domain DTOs, pinned/repository/people/topic/language preview DTOs, tab counts, viewer permission state, `get_public_organization_profile`, and errors for not-found/private/invalid visibility.
- `crates/api/src/routes/organizations.rs`: add `GET /api/orgs/{org}/profile` with signed-cookie viewer detection, standard error envelopes, case-insensitive slug lookup, member/admin visibility rules, and no admin writes.
- `crates/api/src/lib.rs`, route/domain module exports, and `web/src/lib/api.ts` / `web/src/lib/server-session.ts`: wire typed server fetch helpers for later UI phases without changing existing profile helpers.
- `crates/api/tests/organization_profile_contract.rs`: cover public organization shape, verified domains, pinned repository filtering, repository/people/topic/language previews, tab counts, anonymous visibility, member-visible internal data, private repository redaction, case-insensitive slugs, and 404 behavior.

**Verification**: focused `organization_profile_contract` against `TEST_DATABASE_URL` when available, then same-env `make check && make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Organization Header and Overview Shell - replace the placeholder route

**Done**: [x]

**Scope**: Replace `/orgs/{org}` placeholder content with a real Editorial organization overview using the Phase 1 profile contract. The page should render the org avatar/logo, display name, slug, description, verified badge with an explanation link/tooltip, website/social links, follower/member/repository counts, tabs, and non-inert top actions. Sponsor must be hidden or disabled with explicit unavailable state until sponsorships exist.

**Key changes**:
- `web/src/app/orgs/[org]/page.tsx`: fetch `getPublicOrganizationProfile`, derive active tabs from `ORGANIZATION_TABS`, and render the real page when found; keep the existing placeholder/error shell only for unavailable data.
- `web/src/components/OrganizationProfilePage.tsx`: add the Editorial layout using existing primitives (`.card`, `.btn`, `.chip`, `.tabs`, `.av`, type ramp) and patterns from `UserProfilePage`, with `1240px`-style content width and no GitHub palette values.
- `web/src/lib/api.ts` and navigation helpers: add typed org profile DTOs and tab labels/counts while preserving concrete links for repositories, projects, packages, people, teams, and settings.
- `web/tests/organization-profile-page.test.tsx`: assert header identity, verified domain signal, tooltip/explanation link, tab labels/counts, website/social links, sponsor disabled/hidden state, no inert anchors/buttons, and Editorial token usage.
- `ralph/screenshots/build/`: save an initial desktop organization overview screenshot if the seeded data path is available.

**Verification**: focused Vitest for the org overview component, then full `make check && make test`. Browser smoke may be focused to `/orgs/{seed}` if the page has usable seeded data.

---

## Phase 3: Pinned Repositories and Repository Preview - repo cards/rows navigate to real repositories

**Done**: [x]

**Scope**: Fill the main overview content with real pinned repository cards and repository preview rows. Cards and rows must include visibility badges, descriptions, primary language, stars/forks, open issue/PR counts when available, license, updated time, topic chips, and concrete repository links. Empty states must have working CTAs or no CTA if the viewer lacks permission.

**Key changes**:
- Rust org profile query code: add deterministic pin ordering, repository preview sorting by recent activity, repository visibility filtering for anonymous/member/admin viewers, topic/language aggregation, and count fields needed by the UI.
- `web/src/components/OrganizationProfilePage.tsx`: render pinned repository grid and repository preview list using the same dense Editorial repository-card language as profile/repository tabs; long names/descriptions/topics must truncate without layout overflow.
- `web/tests/organization-profile-page.test.tsx`: cover public vs internal/private repository redaction, pin ordering, repository row hrefs, count formatting, topic chips, empty pinned state, empty repository state, and no `href="#"`.
- `web/tests/e2e/organization-profile.spec.ts`: focused browser smoke that opens the overview, clicks pinned/repository rows to real repository pages, validates tab links, and saves `ralph/screenshots/build/orgs-001-phase3-repository-preview.jpg`.

**Verification**: focused Rust contract plus focused Vitest and Playwright smoke, then `make check && make test`. Run `make test-e2e` if the phase adds or changes browser-visible route behavior.

---

## Phase 4: People, Topics, Languages, Sponsoring, and Accessibility - finish overview secondary panels

**Done**: [ ]

**Scope**: Complete the overview sidebar/secondary panels: public people preview, owners/member role indicators where allowed, top languages, most-used topics, sponsoring placeholder/preview, follower count, and accessible verified-domain behavior. Public views show public members only; member/admin views may show internal membership details according to existing permission contracts.

**Key changes**:
- Rust org profile query code: add people preview visibility rules, public member count, viewer membership role, top language totals, top topic counts, sponsorship placeholder state, and follower/sponsor counts if backed by existing tables.
- `web/src/components/OrganizationProfilePage.tsx`: add people avatar rail/list, language summary, topic chips, sponsoring preview/CTA state, and compact responsive placement; all interactive elements must link to concrete org tab routes or be properly disabled.
- `web/tests/organization-profile-page.test.tsx`: cover public/member people visibility, verified tooltip/accessibility labels, language/topic sorting, sponsoring unavailable state, mobile-friendly truncation, keyboard focusable tabs/links, and no dead controls.
- `web/tests/e2e/organization-profile.spec.ts`: add mobile no-overflow smoke and keyboard/tab navigation checks; save `ralph/screenshots/build/orgs-001-phase4-mobile.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright, then `make check && make test`. Run full `make test-e2e` unless the local DB is unavailable; record any DB limitation explicitly.

---

## Phase 5: Final Privacy, Visual, and QA Guardrails - finish orgs-001

**Done**: [ ]

**Scope**: Lock the completed organization overview against regressions and mark `orgs-001.build_pass=true` only after real API data, Editorial UI, navigation, visibility rules, and browser smoke all pass. This phase should not add new product scope; it hardens the finished surface.

**Key changes**:
- `crates/api/tests/organization_profile_contract.rs`: final privacy coverage for anonymous, signed-out, member, admin, and unrelated signed-in viewers; private/internal repository and member redaction; verified-domain shape; count consistency; standard error envelopes with no stack traces.
- `web/tests/organization-profile-page.test.tsx`: final accessibility, no-dead-control, responsive text/truncation, empty-state, disabled-state, and Editorial visual-token guardrails.
- `web/tests/e2e/organization-profile.spec.ts`: desktop and mobile smoke for overview, tabs, verified badge explanation, pinned/repository navigation, people/topics/languages panels, empty states, and no horizontal overflow.
- `ralph/screenshots/build/`: save final desktop, mobile, empty-state, and member-visible screenshots when test data allows.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `orgs-001.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`; browser smoke proves every visible organization overview button, link, row, tab, badge explanation, empty state, and secondary panel has concrete behavior; mandatory Editorial banned-value scan returns zero matches.
