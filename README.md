# opengithub

GitHub clone — Rust API + Next.js frontend, built end-to-end by [Ralph-to-Ralph](https://github.com/namuh-eng/ralph-to-ralph).

Target: `https://github.com/` (repos, PRs, issues, Actions, Pages, Packages, code search, orgs, profiles).
Production hostname: `opengithub.namuh.co`.

## Stack

| Layer | Tech |
|---|---|
| **Backend** | Rust 2021 — Axum + Tokio + SQLx + Tower / Tower-HTTP + Tracing |
| **Frontend** | Next.js + TypeScript (scaffolded by build loop on iteration 1) |
| **Database** | Postgres on AWS RDS, search via `pg_trgm` |
| **Auth** | Native Rust — `oauth2` + `tower-sessions` + `axum-login`, Google OAuth only. Next.js is a thin client. |
| **Cloud** | AWS — ECS Fargate (API), RDS, S3, SES, CloudFront, ECR |
| **DNS** | Cloudflare (zone `namuh.co`) |
| **Loop runtime** | `codex exec --dangerously-bypass-approvals-and-sandbox` (no `claude -p`) |

## Repo layout

```
.
├── Cargo.toml              # Rust workspace root
├── crates/
│   └── api/                # Axum binary `api`
│       ├── src/main.rs
│       └── migrations/     # SQLx migrations
├── web/                    # Next.js + TypeScript (scaffolded by build loop)
├── scripts/
│   ├── preflight.sh        # AWS infra provisioning
│   └── scrape-docs.py      # Doc scraper (Scrapling + trafilatura)
├── target-docs/            # Pre-scraped GitHub docs (3,613 pages + OpenAPI)
├── ralph/                  # Loop orchestration: inspect / architecture / build / QA
├── BUILD_GUIDE.md          # Authoritative stack reference
├── CLAUDE.md               # Project-level constraints
├── ralph-config.json       # Single source of truth for stack decisions
├── prd.json                # Feature list (built by inspect phase)
└── build-spec.md           # Product spec (built by inspect phase)
```

## Commands

All commands route through `make` — never reach for `cargo`/`npx` directly when a target exists.

| Command | What it does |
|---|---|
| `make check` | `cargo check --workspace` + `cargo clippy` + (when `web/`) `tsc --noEmit` + `biome check` |
| `make test` | `cargo test --workspace` + (when `web/`) `vitest run` |
| `make test-e2e` | Playwright (only when `web/` exists) |
| `make all` | `check` + `test` |
| `make dev` | Rust API on `:3016` and Next.js on `:3015` together |
| `make api-dev` | Rust API only |
| `make web-dev` | Next.js only |
| `make build` | Production build for both |
| `make db-migrate` | Run SQLx migrations |
| `make fix` / `make format` | Auto-fix clippy + biome / cargo fmt + biome format |

## Ports

- Rust API: `:3016` (exposes `GET /`, `GET /health` liveness, and `GET /ready` readiness)
- Next.js: `:3015` (exposes public `GET /healthz`)

## Production Docker smoke tests

Build the production images from the repository root:

```bash
docker build -f Dockerfile.api -t opengithub-api:local .
docker build -f Dockerfile.web -t opengithub-web:local .
```

Smoke the API image on port `3016`:

```bash
docker run --rm -p 3016:3016 --name opengithub-api-smoke opengithub-api:local
curl -fsS http://localhost:3016/health
```

Smoke the web image on port `3015`:

```bash
docker run --rm -p 3015:3015 --name opengithub-web-smoke opengithub-web:local
curl -fsSI http://localhost:3015/
```

Runtime secrets such as `DATABASE_URL`, `SESSION_SECRET`, OAuth credentials, and provider API keys must be injected by the orchestrator at container start time. The Docker build context excludes `.env*` files so local secrets are not baked into either image.

## Phases

The clone is built by four sequential autonomous loops, all driven by `codex exec`:

| Phase | Runner | Output |
|---|---|---|
| **1. Inspect** | `ralph/inspect-ralph.sh` (Codex + Ever CLI) | `prd.json`, `build-spec.md`, `sitemap.md`, `target-docs/`, screenshots |
| **1.5. Architecture** | `ralph/architecture-ralph.sh` (Codex) | `ralph/architecture-decisions.json`, finalized `build-spec.md` |
| **2. Build** | `ralph/build-ralph.sh` (Codex) | Source under `crates/api/` and `web/`, tests, migrations |
| **3. QA** | `ralph/qa-ralph.sh` (Codex + Ever CLI) | `qa-report.json`, bug fixes — re-runs Phase 2 on failure |

`ralph/ralph-watchdog.sh` orchestrates all four phases, restarts failed iterations, and commits backups every 30 minutes.

## Running

```bash
# Onboarding already produced ralph-config.json + BUILD_GUIDE.md.
# Kick off the full pipeline:
./ralph/ralph-watchdog.sh

# Or run a phase directly:
./ralph/inspect-ralph.sh https://github.com/
./ralph/build-ralph.sh
./ralph/qa-ralph.sh
```

Loop runs in tmux session `ralph-loop`:

```bash
tmux attach -t ralph-loop      # watch live
tmux capture-pane -t ralph-loop -p | tail -50
```

## Auth setup

`ralph-config.json`: `authMode: "native-rust"`, `authProviders: ["google"]`.

Auth lives in the Rust API. Stack: `oauth2` (Google flow) + `tower-sessions` (Postgres-backed signed cookie) + `axum-login` (extractor). Next.js does not own auth — the sign-in button just hits `GET /api/auth/google/start`.

Google OAuth client config:
- JS origins: `http://localhost:3015`, `https://opengithub.namuh.co`
- Redirect URIs (note: API port `:3016`, not Next.js): `http://localhost:3016/api/auth/google/callback`, `https://opengithub.namuh.co/api/auth/google/callback`

GitHub OAuth is deliberately dropped — even though we're cloning GitHub, opengithub uses Google for sign-in.

## Out of scope

- Paywalls / billing / subscription management
- Payment processing
- GitHub OAuth (we are not federating with the target product)
- GitHub Copilot widgets / agents

## Database

1. Add migration: `crates/api/migrations/<timestamp>_<name>.up.sql` + `.down.sql`
2. Apply: `make db-migrate`
3. Reference from Rust via `sqlx::query!` macros (compile-time checked against `DATABASE_URL`)

## Adding dependencies

**Rust:** edit workspace root `Cargo.toml` `[workspace.dependencies]`, then reference from `crates/api/Cargo.toml` as `dep.workspace = true`.
**Frontend:** `cd web && npm install <pkg>` — pin via `package.json`.

## License

[Elastic License 2.0](./LICENSE) — free to use, modify, and self-host. The only restriction: you cannot offer Opensend as a hosted email service to third parties.

---

<p align="center">
  Built by <a href="https://github.com/jaeyunha">Jaeyun Ha</a> and <a href="https://github.com/ashley-ha">Ashley Ha</a>
</p>

