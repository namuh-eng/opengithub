# Structure Outline: api-003 REST API Rate Limiting and Versioning

**Ticket**: `api-003`
**Date**: 2026-05-03
**Scope**: REST API cross-cutting rate limit headers/versioning plus `/rate_limit` bucket state.

## Phase 1: Cross-cutting middleware and storage

**Done**: [x]

**Actual structure/evidence**:
- Added `crates/api/src/middleware/rate_limit.rs` with global Axum middleware that identifies callers by bearer token hash, session cookie hash, or anonymous client IP; classifies `core` versus `/api/search*` as `search`; and writes `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`, `X-RateLimit-Used`, `X-RateLimit-Resource`, and `X-GitHub-Api-Version` on responses.
- Added Postgres-backed bucket persistence in `crates/api/migrations/202605030042_rate_limit_buckets.up.sql` as an unlogged `rate_limit_buckets` table with `token_id`, `ip`, `resource`, `window_start`, and `request_count`; middleware falls back to process-local buckets when no DB pool is available.
- Wired middleware in `crates/api/src/lib.rs` after request logging so every routed response, including errors, carries the rate/version headers.

## Phase 2: `/rate_limit` endpoint

**Done**: [x]

**Actual structure/evidence**:
- Added `crates/api/src/routes/rate_limit.rs` and route registration for `GET /rate_limit`.
- Response exposes `resources.core`, `resources.search`, and `rate` objects with current `limit`, `remaining`, `reset`, `used`, and `resource` fields for the detected caller bucket.
- `/rate_limit` itself is counted as a `core` request and returns the same rate/version headers as other API routes.

## Phase 3: Client version pin and tests

**Done**: [x]

**Actual structure/evidence**:
- Added `GITHUB_API_VERSION = "2022-11-28"` and `apiRequestHeaders` in `web/src/lib/api.ts`; repository issue API client fetches now pin `X-GitHub-Api-Version`; `web/src/lib/api-docs.ts` documents `/rate_limit` and rate/version response headers.
- Added `crates/api/tests/api_rate_limit_contract.rs` covering response headers/version echo, authenticated bearer tier `5000/hr`, anonymous core tier `60/hr`, search tier `30/min` overage with `403` and `rate_limited` body, and `/rate_limit` JSON shape.
- Updated `web/tests/api-client.test.ts` to assert the pinned version header on a representative API client fetch.

## Verification checklist

**Done**: [x]

- Focused Rust rate-limit contract: `CARGO_INCREMENTAL=0 cargo test -p opengithub-api --test api_rate_limit_contract` passed (4 tests).
- Full required verification recorded in `build-progress.txt` for this ticket.
- `api-003.build_pass=true`, `api-003.needs_structure=false`, and `api-003.qa_pass=false` are set for independent QA handoff.
