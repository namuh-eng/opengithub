# Structure Outline: settings-006 Repository Pages

**Ticket**: `settings-006`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-settings.jsx` (`Pages_A`, `Pages_B`), existing repository settings shell in `web/src/components/RepositorySettingsShell.tsx`, current Pages placeholder in `web/src/app/[owner]/[repo]/settings/pages/page.tsx`, repository settings audit patterns from `settings-001` through `settings-005`, Git ref/tree contracts in `crates/api/src/domain/repositories.rs`, Actions run/artifact contracts in `crates/api/src/domain/actions.rs`, and GitHub Pages references in `target-docs/content/pages/`, `target-docs/content/rest/pages/pages.md`, and `target-docs/content/actions/reference/workflows-and-actions/events-that-trigger-workflows.md`.
**Date**: 2026-05-03

## Phase 1: Pages Settings API Contract - site, source, deployments, domains, and audit

**Done**: [x]

**Scope**: Add the Rust/Postgres contract for repository Pages settings under `/api/repos/{owner}/{repo}/settings/pages`. Repository admins can read and configure Pages publishing source, view deployment status, manage custom domain metadata, request DNS verification, and unpublish the site. Non-admin readers receive only non-sensitive live/status metadata when policy allows it, and private repository behavior remains non-leaky.

**Key changes**:
- `crates/api/migrations/`: add `pages_sites`, `pages_deployments`, `pages_domain_verifications`, and optional `pages_build_artifacts`/job linkage tables if the current job model needs normalized references. Include repository id, publishing source kind (`none`, `branch`, `actions`), branch ref, folder (`/` or `/docs`), default site URL, custom domain, DNS challenge name/value, DNS verification status, HTTPS enforcement, certificate/provisioning status, CloudFront distribution/alias metadata, S3 artifact prefix, unpublished timestamps, and unique constraints for active custom domains.
- Add DTOs for `RepositoryPagesSettings`, `PagesSiteSummary`, `PagesSource`, `PagesDeploymentSummary`, `PagesDomainState`, `PagesDnsChallenge`, `PagesMutation`, `PagesUnpublishResult`, and structured validation errors.
- `GET /api/repos/{owner}/{repo}/settings/pages`: admins receive editable source/domain/deployment data, available branch refs, valid folder options derived from Git tree contents, recent deployments, Actions workflow/template suggestions, and warnings. Non-admin behavior must avoid exposing private custom-domain verification details.
- Mutation endpoints: update publishing source, save/remove custom domain, toggle HTTPS enforcement, request/recheck DNS verification, trigger a branch-source deployment, connect an Actions artifact deployment, and unpublish Pages without deleting repository code.
- Validate that selected branches exist in `repository_git_refs`, folders are limited to root or `/docs` and exist at the selected commit when required, Actions source has compatible workflow/artifact metadata when linking, custom domains are normalized and unique, wildcard/unsupported domains are rejected, HTTPS cannot be enforced until domain/certificate prerequisites are ready, and archived repositories block writes.
- Successful writes insert `repository_settings_audit_events` with event types like `repository.pages.source.update`, `repository.pages.domain.save`, `repository.pages.domain.remove`, `repository.pages.https.update`, `repository.pages.deploy.request`, and `repository.pages.unpublish`.
- Add `crates/api/tests/repository_pages_settings_contract.rs` covering admin gates, private repository privacy, branch/folder validation, Actions-source validation, domain normalization/conflict checks, DNS challenge generation, HTTPS prerequisite blocking, unpublish semantics, deployment row creation, audit rows, and standard error envelopes without stack/env leakage.

**Verification**: focused `repository_pages_settings_contract` against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && DB_SSL=false make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Pages Settings Shell - source, domain, status, and deployment history

**Done**: [x]

**Scope**: Replace the `/[owner]/[repo]/settings/pages` placeholder with a real Editorial Pages settings surface backed by the Phase 1 API. The page must cover disabled/empty state, branch publishing, Actions publishing, live site status, custom domain status, deployment history, forbidden/unavailable states, and responsive layout without GitHub/Primer visual regression.

**Key changes**:
- `web/src/lib/api.ts` and server-session helpers: add typed Pages settings DTOs and cookie-backed fetch helpers preserving forbidden/unavailable result states.
- `web/src/app/[owner]/[repo]/settings/pages/page.tsx`: fetch the settings contract server-side and render concrete content inside `RepositorySettingsShell`.
- Add `web/src/components/RepositoryPagesSettingsPage.tsx` with a build/deployment card, Source selector (`None`, `Deploy from a branch`, `GitHub Actions`), branch dropdown, folder dropdown for `/(root)` and `/docs`, Save button, workflow template suggestions for Actions source, latest deployment link, live-site callout, and recent deployment pipeline/status rows.
- Add a custom domain card with domain input, Save/Remove buttons, DNS verification status, displayed challenge record, Recheck DNS control, HTTPS enforcement toggle, certificate/provisioning chips, and warnings for misconfigured DNS or takeover risk.
- Non-admin/forbidden states must not leak private domain challenge values, CloudFront aliases, or S3 storage keys. Unavailable states must offer concrete retry/navigation paths.
- Use only Editorial primitives and tokens: `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.t-label`, `.t-mono-sm`, `var(--ink-*)`, `var(--line)`, `var(--accent)`, and semantic chips. Do not introduce GitHub colors, Octicons, Primer imports, nested cards, or dead `href="#"` links.
- Add focused `web/tests/repository-pages-settings-page.test.tsx` coverage for disabled, branch, Actions, live, domain pending/verified/error, forbidden/unavailable states, no inert anchors/buttons, no private metadata leakage, and Editorial primitive/token usage.

**Verification**: focused Vitest for the page, mandatory Editorial banned-value scan, then `make check && make test`. Save a browser screenshot if the E2E seed already exposes a repository with Pages fixtures.

---

## Phase 3: Source and Domain Mutations - confirmed forms, DNS checks, and HTTPS controls

**Done**: [x]

**Scope**: Wire every visible Pages settings action to real server-confirmed writes. Admins must be able to configure source, trigger deployment, manage custom domains, recheck DNS, toggle HTTPS when eligible, and unpublish Pages with inline success/error feedback. Invalid inputs must never update local UI optimistically.

**Key changes**:
- Add same-origin Next.js route handlers or server actions under the Pages settings route that forward authenticated mutations to the Rust API without adding JS-side auth.
- Implement source form behavior: Source selector changes reveal the correct branch/folder or Actions template controls; Save validates locally and through API; saved branch source enqueues a Pages deployment job; saved Actions source shows template guidance and links to relevant workflow routes.
- Implement custom domain form behavior: Save normalizes input, shows the returned DNS challenge, blocks verified UI until server verification succeeds, supports Remove with confirmation, and refreshes state only from confirmed API responses.
- Implement DNS recheck using Cloudflare-aware backend verification where credentials are available, with a provider-neutral fallback that records `pending`/`misconfigured` instead of faking success. Surface TTL/propagation warnings as non-blocking Editorial callouts.
- Implement HTTPS enforcement toggle as a confirmed mutation that is disabled with a reason until domain verification and certificate/provisioning state allow it.
- Implement Unpublish Pages confirmation that disables serving/deployments and removes CloudFront/S3 publication metadata while preserving repository source files and historical deployment rows.
- Extend Rust contract tests for source update, deployment enqueue, domain save/remove, Cloudflare/DNS verification outcomes, HTTPS prerequisite enforcement, unpublish audit rows, archived repository blocking, duplicate active domain conflict, and no optimistic state assumptions.
- Extend Vitest coverage for all forms, validation errors, loading/success/error states, confirmation dialogs, disabled controls with reasons, no local-only updates, and no dead controls.
- Add `web/tests/e2e/repository-settings-pages.spec.ts`: seed an admin repository with refs/docs folder, configure branch source, verify deployment queued after reload, configure a pending custom domain and recheck DNS, verify HTTPS disabled until eligible, unpublish a disposable site, verify forbidden non-admin state, and save `ralph/screenshots/build/settings-006-phase3-pages-mutations.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` when local database/dev servers are stable.

---

## Phase 4: Pages Build and Serving Integration - branch artifacts, Actions artifacts, CloudFront/S3, and status

**Done**: [x]

**Scope**: Make Pages operational beyond settings rows. Branch-source and Actions-source deployments should create durable static artifacts, update deployment status, expose a live/public URL state, and model CloudFront/S3/Cloudflare integration without deleting repository code.

**Key changes**:
- Add or extend a Rust worker/job module for `pages-build-deploy` leases that reads queued deployments, resolves branch/folder source from Git metadata, copies static files into the configured S3 prefix or local test storage abstraction, records artifact manifests/checksums, and marks deployment status as queued/building/deployed/failed.
- For branch publishing, validate source folder contents, include `CNAME` handling when a custom domain is configured, preserve not-found/error metadata for missing source folder, and keep build logs bounded/redacted.
- For Actions publishing, connect successful workflow artifacts intended for Pages deployment to `pages_deployments`, record workflow run/artifact ids, and avoid inventing deployments from unrelated artifacts.
- Add a serving/status contract: deployment URLs, default project URL, optional custom domain URL, CloudFront distribution/alias metadata, cache invalidation status, and degraded status when cloud resources are not provisioned locally.
- Integrate with existing webhook/activity/notification foundations where available: Pages deployment events should be visible to webhook delivery subscriptions and repository activity without blocking Phase 4 on unsupported event families.
- Add provider health checks or preflight hooks for S3/CloudFront/Cloudflare env vars when running against real AWS; local tests may use mocked storage clients but must keep production metadata shape.
- Extend Rust tests for branch artifact publication, missing `/docs` failure, Actions artifact linkage, unpublish disabling serving, deployment status transitions, storage key bounds, CloudFront alias gating until DNS verified, webhook/activity event enqueue, and no source-code deletion.
- Add frontend assertions that deployment history and status callouts show worker-recorded state, failed build reasons, artifact/source metadata, and live URL links only when confirmed.

**Verification**: focused worker/domain tests, focused Pages settings contract, focused webhook/activity integration tests where available, then `make check && make test`. Run focused Playwright for Pages settings plus Actions run/deployment linkage when seeded data is available.

---

## Phase 5: Guardrails, API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [x]

**Scope**: Finish `settings-006` as a complete vertical slice only after API, UI, mutation flows, deployment worker behavior, docs, screenshots, and QA handoff are verified. This phase should not add unrelated repository settings areas such as protected tags, security analysis, organization Pages policies, packages, or wiki.

**Key changes**:
- Finalize API docs in `web/src/lib/api-docs.ts` for Pages settings read/update, source changes, deployment trigger/status, domain save/remove, DNS recheck, HTTPS toggle, and unpublish endpoints, including auth/visibility, validation errors, audit behavior, Cloudflare/CloudFront/S3 notes, and degraded local-mode behavior.
- Extend `qa-hints.json` with deeper QA targets: real AWS S3/CloudFront publication, real Cloudflare DNS verification, ACM/certificate readiness, domain takeover prevention, custom-domain removal and CNAME behavior, cache invalidation, Actions artifact provenance, missing `/docs` branch source failures, concurrent admin edits, private repository leakage, and accessibility traversal.
- Ensure every visible button/link/form has concrete behavior or an accessible disabled state; verify keyboard navigation through Source selector, branch/folder controls, domain form, DNS recheck, HTTPS toggle, unpublish confirmation, and deployment history links.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/settings-006-final-pages-*.jpg` for disabled, branch-source, Actions-source, domain pending/verified/error, live deployment, mobile, and forbidden states.
- Run the mandatory Editorial banned-value scan before commit and fix any touched-file regressions.
- Update `build-progress.txt`, `.qrspi/settings-006/structure.md`, and `prd.json`; set `settings-006.build_pass=true` only after all implementation phases are complete and verified; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when available, browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
