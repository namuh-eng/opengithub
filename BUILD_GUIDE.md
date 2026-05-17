# BUILD_GUIDE.md — opengithub

Authoritative stack reference for the build loop. Read this before running any commands or assuming framework conventions. If anything in `ralph/build-prompt.md` or `CLAUDE.md` contradicts this file, **this file wins**.

## Stack
- **Backend**: Rust 2021 — Axum (HTTP), Tokio (runtime), SQLx (Postgres), Tower / Tower-HTTP (middleware), Tracing (logs).
- **Frontend**: Next.js + TypeScript — to be scaffolded at `web/` on the build loop's first iteration.
- **Database**: Postgres on AWS RDS. SQLx migrations at `crates/api/migrations/`. Search via `pg_trgm`.
- **Auth**: Native Rust — `oauth2` crate (Google OAuth code flow) + `tower-sessions` (Postgres-backed signed cookie sessions) + `axum-login` (extractor). No Better Auth, no JS-side auth library. Next.js is a thin client.
- **Cloud**: AWS — ECS Fargate (Rust API), RDS Postgres, S3, SES, CloudFront, ECR. DNS on Cloudflare (zone `namuh.co`).

## Auth env contract
- Full production contract: `docs/production-runtime-env.md`.
- `APP_ENV=staging|production` (or `ENVIRONMENT`) enables fail-fast production validation.
- `APP_URL` / `PUBLIC_APP_URL`: browser-facing Next.js origin, local default `http://localhost:3015`; HTTPS required in staging/production.
- `API_URL`: Rust API origin used for OAuth callbacks, local default `http://localhost:3016`; HTTPS required in staging/production.
- `PORT`: Rust API listen port; defaults to `3016`. Web `npm start` uses `PORT`, defaulting to `3015`.
- `AUTH_GOOGLE_ID`, `AUTH_GOOGLE_SECRET`: Google OAuth client credentials; required in staging/production.
- `SESSION_SECRET`: signing secret for OAuth state and the later session cookie; required in staging/production.
- `SESSION_COOKIE_NAME`: defaults to `__Host-session`.
- `SESSION_COOKIE_SECURE`: defaults to `false` for localhost and `true` for deployed environments; must be `true` in staging/production.

## Git transport env contract (git-001)
- `OPENGITHUB_GIT_STORAGE_DIR`: local bare-repository root for development and tests. Defaults to `${TMPDIR}/opengithub-git-storage`.
- Production ECS tasks should mount or provision durable Git object storage. The current Phase 1 implementation records `repository_git_storage.storage_kind='local_bare'`; later S3-backed phases should preserve the same table contract while moving object bytes under the AWS git storage prefix.

Google OAuth redirect URIs:
- Local: `http://localhost:3016/api/auth/google/callback`
- Production: `https://opengithub.namuh.co/api/auth/google/callback`

## AI provider env contract (ai-001)
- `OPENAI_API_KEY`: server-side OpenAI key used by the Rust API for ai-001 (AI repo summary, AI PR summary tab, AI changelog). Never expose to the browser. Same provider already used by the Codex build loop.
- Endpoint: `https://api.openai.com/v1/chat/completions`.
- Models: `gpt-4o-mini` for cheap calls (repo summary, hover blurbs); `gpt-4o` for PR summaries and changelog generation.
- Per-user calls are rate-limited via the `api-003` buckets. Caching keyed on `(content_hash, prompt_version, model)`.
- `scripts/deploy.sh` MUST pass `OPENAI_API_KEY` through to the ECS Fargate task definition alongside `DATABASE_URL`, `SESSION_SECRET`, etc. Treat it like a secret (Secrets Manager / SSM Parameter Store, not a plain env var in the task definition).

## Repo layout
```
.
├── Cargo.toml             # workspace root
├── crates/
│   └── api/               # Axum HTTP API (binary: api)
│       ├── Cargo.toml
│       ├── src/main.rs
│       └── migrations/    # SQLx migrations (create when DB schema needed)
├── web/                   # Next.js + TypeScript (build loop scaffolds this)
│   ├── package.json
│   ├── src/app/...
│   └── tests/             # Vitest unit + Playwright e2e
├── scripts/
│   └── preflight.sh       # AWS infra provisioning
├── Makefile               # contract: see commands below
├── ralph-config.json      # single source of truth for stack decisions
└── .env                   # local secrets (gitignored)
```

## Commands (use `make` — never reach for `npx`/`cargo` directly when a make target exists)

| Command | What it does |
|---|---|
| `make check` | `cargo check --workspace` + `cargo clippy` + (when web/ exists) `tsc --noEmit` + `biome check` |
| `make test` | `cargo test --workspace` + (when web/ exists) `vitest run` |
| `make test-e2e` | Playwright (only when web/ exists) |
| `make all` | `make check && make test` |
| `make dev` | Rust API on `:3016` and Next.js on `:3015` together |
| `make api-dev` | Rust API only |
| `make web-dev` | Next.js only |
| `make build` | Production build for both |
| `make db-migrate` | Run SQLx migrations from `crates/api/migrations/` |
| `make fix` | Auto-fix clippy + biome |
| `make format` | `cargo fmt --all` + biome format |
| `make clean` | `cargo clean` + remove `web/.next`, web caches |

## Ports
- **Rust API**: `:3016`
- **Next.js**: `:3015`

The Rust API exposes `GET /` and lightweight `GET /health` for liveness, plus `GET /ready` for production readiness. `/ready` returns HTTP 200 only when RDS/Postgres is reachable and non-2xx when critical dependencies are unavailable. Next.js exposes public `GET /healthz` without auth. AWS ALB target groups should use `/ready` for the API target group and `/healthz` for the web target group; ECS container health checks should use `/health` for API liveness and `/healthz` for web liveness. The machine-readable contract lives in `deploy/aws/health-checks.json`, and `scripts/deploy.sh` waits on `/ready` and `/healthz` before declaring rollout success.

## Adding a Rust dependency
Edit `Cargo.toml` (workspace root) — add the dep to `[workspace.dependencies]` first, then reference it from `crates/api/Cargo.toml` as `dep.workspace = true`. This keeps versions in sync across crates.

## Adding a frontend dependency
`cd web && npm install <pkg>`. Always pin via `package.json`.

## DB workflow
1. Add a migration: `crates/api/migrations/<timestamp>_<name>.up.sql` + `.down.sql`.
2. Apply: `make db-migrate`.
3. Reference tables/columns from Rust via `sqlx::query!` macros (compile-time checked against `DATABASE_URL`).

## Scaffolding the Next.js frontend (build loop, first iteration)
```bash
npx create-next-app@latest web \
  --ts --tailwind --eslint --app --src-dir \
  --import-alias "@/*" --use-npm
cd web
npm install -D vitest @vitest/ui @playwright/test biome
```
Then update `web/package.json` scripts: `dev`, `build`, `start`, `test`, `test:e2e`, `lint`, `format`. Add `playwright.config.ts` and `vitest.config.ts`.

## Loop runtime
All ralph loops (inspect, architecture, build, QA) run via `codex exec` — the OpenAI Codex CLI. Prompts in `ralph/*.sh` inline file contents (codex doesn't honor `@filename`). The helper `ralph/lib/inline.sh` provides `inline_files <paths...>`.

## Deployment pipeline

Production and staging ECS rollouts are driven by `.github/workflows/deploy.yml`, which invokes `scripts/deploy.sh`. The deploy action builds `Dockerfile.api` and `Dockerfile.web`, tags both images with the current git SHA, pushes to ECR, runs SQLx migrations before rollout, registers ECS task definitions, updates API/web ECS services, waits for ECS stability, verifies API `/ready` and web `/healthz`, and invalidates CloudFront when `CLOUDFRONT_DISTRIBUTION_ID` is set. Use `DRY_RUN=1` for local/staging command-construction verification without AWS credentials.

Rollback uses `scripts/deploy.sh rollback <environment>` with `ROLLBACK_API_TASK_DEFINITION` and `ROLLBACK_WEB_TASK_DEFINITION` set to the previous known-good ECS task definition ARNs. See `docs/deployment.md` for exact staging deploy and rollback commands.
