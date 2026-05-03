# Structure Outline: packages-003 OCI Container Registry APIs

**Ticket**: `packages-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, existing package detail/list/settings contracts in `crates/api/src/domain/packages.rs`, repository package metadata routes in `crates/api/src/routes/packages.rs`, PAT validation in `crates/api/src/domain/tokens.rs`, package schema migrations through `202605031930_package_detail_metadata`, docs in `target-docs/content/packages/working-with-a-github-packages-registry/working-with-the-container-registry.md`, and Actions package docs in `target-docs/content/packages/managing-github-packages-using-github-actions-workflows/`.
**Date**: 2026-05-03

## Phase 1: OCI Registry Auth, Routing, and Manifest Read Contract

**Done**: [x]

**Scope**: Add the minimal Docker Registry HTTP API v2 surface for authentication challenges and manifest reads without yet accepting blob uploads. Docker/OCI clients should receive correct `WWW-Authenticate` challenges, PAT or workflow-token bearer credentials should map to package read permission, public packages should pull anonymously, and private/internal packages should avoid metadata leaks.

**Key changes**:
- `crates/api/migrations/`: add registry protocol metadata that is missing from the package detail schema, such as manifest media type, config digest, tag/digest aliases, manifest byte size, registry audit rows, and any token/session table needed for short-lived registry bearer challenges.
- `crates/api/src/domain/packages_registry.rs` or a focused module under `domain/packages.rs`: add OCI reference parsing for `/v2/{namespace}/{image}/...`, digest validation, manifest media-type negotiation, PAT/workflow-token auth resolution, package read/write/admin capability checks, and redaction-safe errors.
- `crates/api/src/routes/packages.rs`: mount `/v2/`, `/v2/:namespace/:image/manifests/:reference`, and supporting challenge endpoints while preserving existing repository and owner package routes.
- `crates/api/src/domain/tokens.rs`: reuse hashed `personal_access_tokens` validation for registry Basic/Bearer auth, with required package scopes enforced instead of session-cookie auth.
- `crates/api/tests/package_registry_contract.rs`: cover `/v2/` health/challenge, anonymous public manifest read, private manifest denial, PAT read success, invalid token denial, malformed digest/tag validation, Accept negotiation for OCI and Docker manifest media types, and no `storage_key` leakage.

**Verification**: focused `DB_SSL=false CARGO_INCREMENTAL=0 cargo test -p opengithub-api --test package_registry_contract`, mandatory Editorial banned-value scan, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Blob Upload, Manifest Push, Tag Listing, and Download Accounting

**Done**: [x]

**Scope**: Implement real container push and pull byte flows. Clients must upload blobs by digest, complete uploads only after checksum validation, push manifests that create or update package versions/tags, list tags, pull blobs with permission checks, and increment `package_downloads` only on actual blob/manifest transfer.

**Key changes**:
- `crates/api/migrations/`: add OCI upload session rows with UUID upload IDs, expected digest, byte size, storage kind/key, expiry/cancel state, plus indexes for package blob digest uniqueness and tag lookup.
- `crates/api/src/domain/packages_registry.rs`: implement blob upload start/patch/complete/cancel, local development storage under an env-driven package registry storage root, S3-compatible storage-key fields for production handoff, manifest validation against uploaded layers/config, tag upsert, digest immutability checks, and download counter/event recording.
- `crates/api/src/routes/packages.rs`: expose `POST/PATCH/PUT/DELETE /v2/{namespace}/{image}/blobs/uploads/`, `GET/HEAD /v2/{namespace}/{image}/blobs/{digest}`, `PUT/GET/HEAD /v2/{namespace}/{image}/manifests/{reference}`, and `GET /v2/{namespace}/{image}/tags/list` with Docker-compatible status codes and headers.
- Existing detail/list/settings DTOs: make package pages reflect registry-published versions, digests, platforms, download counts, and storage availability without exposing raw storage keys.
- `crates/api/tests/package_registry_contract.rs`: cover resumable uploads, checksum mismatch rejection, duplicate blob reuse, manifest push creating package/version/blob rows, tag overwrite semantics, tag listing pagination, manifest/blob pull accounting, private pull denial, and malformed client behavior.

**Verification**: focused Rust registry contract tests, `cargo fmt --all --check`, `cargo check -p opengithub-api`, mandatory banned-value scan, then full `make check && make test`. Add a `.scratch/` or scripted curl scenario using real API routes and local storage if Docker CLI is unavailable; do not commit `.scratch/`.

---

## Phase 3: Actions Publishing Integration and Repository Linking

**Done**: [ ]

**Scope**: Connect registry publishing to opengithub Actions semantics. Workflow tokens should be able to publish packages for the repository that produced the run, package versions should link back to workflow runs/jobs when available, OCI source labels should connect packages to repositories, and webhook/activity/audit events should be emitted for publish and pull actions.

**Key changes**:
- `crates/api/src/domain/actions.rs` and related workflow token code: add a bounded workflow package token model or reuse existing run/job identity so `GITHUB_TOKEN`-style credentials can request `packages:read` or `packages:write` only for the owning repository.
- `crates/api/src/domain/packages_registry.rs`: parse `org.opencontainers.image.source`, description, licenses, revision, and URL annotations from manifests/config; auto-link package to a repository when the workflow context or source label resolves to a readable repository; inherit default package permissions from the repository for workflow-created packages.
- `crates/api/src/domain/webhooks.rs` and activity/audit tables: enqueue package publish/delete/download events, record actor provenance (`pat`, `workflow`, or anonymous public pull), and keep immutable audit rows for tag updates.
- `web/src/lib/api-docs.ts`: add early developer docs for `docker login`, workflow-token examples, source labels, and permission expectations as soon as the backend contract exists.
- Tests: extend registry and Actions contract tests for workflow token publish, repository auto-link, label-based link fallback, package permission inheritance, webhook queue rows, activity rows, and fork/private read boundaries.

**Verification**: focused `package_registry_contract` plus any Actions workflow-token contract test, docs Vitest if docs change, mandatory banned-value scan, then `make check && make test`.

---

## Phase 4: Package Admin Mutations, Delete/Restore, and Settings Enablement

**Done**: [ ]

**Scope**: Replace the disabled package settings placeholders from `packages-002` with real admin mutations needed by registry lifecycle: visibility changes, package access grants, linked repository updates, package/version delete and restore, and safe tag/manifest deletion behavior that preserves audit history.

**Key changes**:
- `crates/api/migrations/`: add soft-delete/restore metadata for packages, package versions, tags/manifests, and blobs where absent, plus audit-event detail columns needed for admin changes.
- `crates/api/src/domain/packages.rs` and `packages_registry.rs`: add admin-gated visibility, access grant/revoke, repository link/unlink, delete/restore package, delete/restore version or tag, and garbage-collection eligibility helpers. Do not physically delete blobs in this phase unless the storage provider confirms safe retention behavior.
- `crates/api/src/routes/users.rs`, `routes/organizations.rs`, and registry routes: expose same-origin settings mutation endpoints for the app and Docker-compatible delete endpoints for registry clients.
- `web/src/components/PackageSettingsPage.tsx` and same-origin action routes: turn disabled settings controls into server-confirmed Editorial controls with success/error feedback, confirmation for destructive operations, and redaction-safe non-admin states.
- Tests: cover admin/non-admin boundaries, visibility transitions, grant/revoke, linked repository changes, delete/restore preserving audit/download history, Docker delete semantics, UI mutation rollback on API error, and no dead controls.

**Verification**: focused Rust package settings/registry tests, focused `web/tests/package-detail-page.test.tsx`, focused Playwright `package-detail.spec.ts` for settings mutations, mandatory banned-value scan, then `make check && make test`; run `make test-e2e` when the local migration state can run.

---

## Phase 5: Developer Docs, Docker Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `packages-003` only after the registry API, auth, push/pull, Actions publishing, admin lifecycle, docs, and QA handoff are verified. This phase should prove a real primary flow with Docker-compatible HTTP calls or the Docker CLI against the local API/storage path.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document the OCI registry endpoints, auth challenges, PAT scopes, workflow-token publishing, docker login/push/pull/tag-list/digest-pull snippets, delete/restore behavior, webhook/activity side effects, local vs S3 storage expectations, and security/redaction rules.
- `qa-hints.json`: append package registry QA targets covering real Docker CLI compatibility, S3 storage health, concurrent blob uploads, upload resume/cancel, large layers, manifest-list/multi-arch images, private/internal leakage, workflow-token permissions, source-label repository linking, webhook delivery, delete/restore retention, and audit exports.
- `web/tests/e2e/package-detail.spec.ts` or a dedicated registry smoke: verify docs snippets render, package detail pages reflect registry-published versions, settings mutations survive refresh, no dead controls remain, and screenshots are saved under `ralph/screenshots/build/` for docs/settings/detail states.
- `.scratch/` scenario or committed integration test fixture: run a real push/pull/tag-list flow against local storage and API auth without mocks; keep any transient artifacts ignored.
- `build-progress.txt`, `.qrspi/packages-003/structure.md`, and `prd.json`: record verification evidence and set `packages-003.build_pass=true` only after all phases are complete and verified; leave `qa_pass=false`.

**Verification**: focused Rust registry tests, focused docs/Vitest and Playwright checks, mandatory Editorial banned-value scan, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, same-env `make test`, and `make test-e2e` when migration state allows. If the shared database still has the known duplicate `202605030041` migration checksum issue, document it precisely and include a fresh-DB Playwright or Docker-compatible HTTP scenario pass.
