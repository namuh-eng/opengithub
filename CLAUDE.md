# opengithub — Autonomous GitHub Clone (Rust + Next.js)

## What This Is
A four-phase autonomous system cloning **github.com** from a URL.
Phase 1: Inspect (Codex + Ever CLI) → Phase 1.5: Architecture (Codex) → Phase 2: Build (Codex) → Phase 3: QA (Codex + Ever CLI)

All loops run via `codex exec` — no Anthropic API key required.

## Tech Stack
- **Backend**: Rust 2021 (Axum + Tokio + SQLx). Workspace root `Cargo.toml`, API crate at `crates/api/`.
- **Frontend**: Next.js + TypeScript in `web/` (scaffolded by build loop on first iteration).
- **Database**: Postgres on AWS RDS. Search via Postgres + `pg_trgm` (no separate code indexer for now).
- **Auth**: Native Rust — `axum-login` + `oauth2` crate + `tower-sessions` (Postgres-backed cookie sessions), Google OAuth only. `authMode: "native-rust"` in `ralph-config.json`.
- **Cloud**: AWS — ECS Fargate (Rust API), RDS Postgres, S3 (git/packages/releases), SES (email), CloudFront (CDN), ECR.
- **DNS**: Cloudflare (zone `namuh.co`); production hostname `opengithub.namuh.co`.
- **Testing**: `cargo test` (Rust), Vitest + Playwright (Next.js).
- **Linting**: `cargo clippy` + `rustfmt` (Rust), Biome (Next.js).

## Commands
All commands go through `make`. The Makefile is a contract — onboarding wires up real recipes based on your stack.
- `make doctor` — diagnose local verification stack (run this FIRST in any new worktree)
- `make setup-local` — boot Docker, start test Postgres on :55433, apply migrations
- `make check` — typecheck + lint/format
- `make test` — run unit tests
- `make test-e2e` — run E2E tests (requires `make setup-local` once, then dev server)
- `make all` — check + test
- `make dev` — start dev server on port 3015
- `make build` — production build
- `make db-push` — push schema to database

## Verification Loop (CRITICAL — read before claiming "tests pass")

**Every worktree** needs the local test DB running before E2E or DB-backed tests are real.

1. **In a fresh worktree, run `make doctor` first.** It prints a green/red checklist (Docker, postgres-test container, .env.test, .env).
2. **If anything is red, run `make setup-local`.** It's idempotent — boots Docker if needed, starts the postgres-test container, runs migrations.
3. **Then run `make all && make test-e2e`.** Use the committed `.env.test` (`TEST_DATABASE_URL=postgresql://opengithub:opengithub@localhost:55433/opengithub_test`). Do NOT invent a `TEST_DATABASE_URL` — the watchdog wasted many cycles doing that.
4. **`make test-e2e` exiting with "no Playwright detail" means the DB was unreachable** — go back to step 2. Do not log this as "verified".

The test DB definition is in `docker-compose.test.yml` (port 55433, user/pass `opengithub`/`opengithub`, db `opengithub_test`). `.env.test` is committed and matches it.

## Worktrees
Use `./hack/create_worktree.sh [name] [base]`. It symlinks `.env`, `.env.test`, `.mcp.json`, copies `.claude/`, touches `.ralph-setup-done`, and runs `make doctor` so you immediately know if setup is healthy.

## Quality Standards
- Strict type checking enabled (language-specific: TypeScript strict, Go vet, etc.)
- Every feature must have at least one unit test AND one E2E test
- Run `make check && make test` before every commit
- Small, focused commits — one feature per commit

## Architecture
- `Cargo.toml` — Rust workspace root.
- `crates/api/` — Axum API service (binary: `api`). All Rust code lives under `crates/`.
- `web/` — Next.js + TypeScript frontend. Scaffolded by the build loop on first iteration if absent.
- `crates/api/migrations/` — SQLx migrations (created by build loop when DB schema is needed).
- `web/tests/` — Vitest unit tests.
- `web/tests/e2e/` — Playwright E2E tests.
- `scripts/` — infrastructure provisioning (`preflight.sh` for AWS) and deployment.

## Pre-configured (DO NOT reinstall or recreate)
- **Makefile** — `make check`, `make test`, `make test-e2e`, `make all` (contract targets)
- **hack/run_silent.sh** — output formatting helper used by Makefile

## Stack Setup
- Onboarding wrote `ralph-config.json` (single source of truth for stack decisions).
- `setup-stack.sh` was **skipped** — no `rust-platform` template ships with the onboarding skill. The Rust workspace was scaffolded directly during onboarding (`Cargo.toml`, `crates/api/Cargo.toml`, `crates/api/src/main.rs`).
- `.ralph-setup-done` contains the marker `rust-platform-custom`.
- Build loop is responsible for scaffolding `web/` (Next.js + TypeScript) on its first iteration.

## Environment
- **Cloud CLI** — configure via onboarding (AWS, Vercel, GCP, Azure)
- **`.env`** — copy from `.env.example` and fill in your values

## Authentication
`authMode: "native-rust"`, `authProviders: ["google"]`.
- Auth lives entirely in the Rust API. Stack: `oauth2` (Google OAuth flow) + `tower-sessions` (signed cookie sessions, Postgres store) + `axum-login` (extractor / middleware).
- Endpoints: `GET /api/auth/google/start` (issues redirect to Google), `GET /api/auth/google/callback` (exchanges code, upserts user, sets session cookie), `POST /api/auth/logout`, `GET /api/auth/me`.
- Next.js is a thin client — it does not own auth. Sign-in button hits `/api/auth/google/start`. Session is a `__Host-session` cookie, `HttpOnly`, `Secure`, `SameSite=Lax`.
- Schema (Rust-owned): `users`, `oauth_accounts (provider, provider_user_id, user_id)`, `sessions (id, user_id, expires_at, data)`. Migrations under `crates/api/migrations/`.
- Cookie signing key: `SESSION_SECRET` in `.env` (32-byte base64).
- OAuth client config:
  - JS origins: `http://localhost:3015`, `https://opengithub.namuh.co`
  - Redirect URIs: `http://localhost:3016/api/auth/google/callback`, `https://opengithub.namuh.co/api/auth/google/callback`
- Auth is **P1 priority** — build it before core features.

## Out of Scope — DO NOT build
- Paywalls, billing, subscription management
- Payment processing
