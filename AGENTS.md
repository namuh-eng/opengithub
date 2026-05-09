# Design System — READ THIS BEFORE WRITING ANY UI

**The product clones GitHub's *capabilities*, NOT GitHub's visual design.** Do NOT use GitHub blue/green/red, Octicons, Primer chrome, or chunky borders. The visual system is **Editorial** — a calm, magazine-grade design.

- **Reference**: `design/project/Prototype.html` (loader), `design/project/og.css` (tokens), `design/project/og-screens-*.jsx` (11 hi-fi screens — Landing, Login, Dashboard, Notifications, Repo, File, Pulls, PR detail, PR diff, Issues, Actions). Read these before designing or implementing any UI.
- **Tokens (live)**: `web/src/app/og.css` and `web/src/app/og-themes.css` — imported globally via `web/src/app/globals.css`. All UI must use these CSS variables (`var(--ink-1)`, `var(--accent)`, `var(--display)`, etc.), NOT hardcoded hex.
- **Type**: Fraunces (display), Inter Tight (body/sans), JetBrains Mono (code) — loaded via `next/font` in `web/src/app/layout.tsx` and exposed as `--font-fraunces` / `--font-inter-tight` / `--font-jetbrains-mono`.
- **Component primitives**: `.btn` / `.chip` / `.card` / `.input` / `.av` / `.tabs` / `.list-row` / `.kbd` / `.palette` are pre-styled in `og.css`. Use them. Do not roll your own.
- **Type ramp**: `.t-display`, `.t-h1`, `.t-h2`, `.t-h3`, `.t-body`, `.t-sm`, `.t-xs`, `.t-mono`, `.t-label`, `.t-num`. Use these classes — do not pick arbitrary font-sizes.
- **Single accent rule**: only `--accent` (rust) for active/selected/primary; semantic chips for state (`.chip.ok`, `.chip.warn`, `.chip.err`).
- **What NOT to do**: see `web/AGENTS.md` "What NOT to do" section.

For full design rules, tokens, and component contracts, read `web/AGENTS.md` "Design System" section.

---

# Ralph-to-Ralph: QA Agent Guide

## Your Role
You are the independent QA evaluator. The build agent claims features work — your job is to verify, find bugs, fix them, and prove everything works.

## What This Is
An autonomously-built clone of a SaaS product. It has its own backend (cloud services + database) and may be deployed. Your job is to make sure it actually works.

## Commands
- `make doctor` — diagnose local verification stack. **Run this FIRST in any worktree.**
- `make setup-local` — start Docker, bring up the test Postgres on :55433, apply migrations. Idempotent.
- `make check` — typecheck + lint/format. Run after every code change.
- `make test` — run unit tests. Must all pass.
- `make test-e2e` — run E2E tests. Requires `make setup-local` to have run.
- `make all` — check + test
- `make dev` — start dev server (if not already running)

## Shared Rust build cache

The Makefile routes Cargo through `hack/cargo_locked.sh`, which defaults to:

- `CARGO_TARGET_DIR=$HOME/.cache/opengithub/cargo-target`
- `CARGO_BUILD_JOBS=2`
- `CARGO_INCREMENTAL=0`

This keeps large Rust build artifacts out of individual worktrees and serializes Cargo invocations with a repo-scoped lock. Do not run raw `cargo clean` in a QA worktree; it can wipe the shared target cache for every active lane. If you need custom Cargo commands, prefer `./hack/cargo_locked.sh <subcommand> ...`.

## Verification Setup — read before claiming a test pass

If `make test-e2e` reports "no Playwright detail" or DB-backed tests "self-skip", the local test DB is not running. **Do not log this as "verified".** Fix the setup:

1. `make doctor` — green/red checklist of what's missing.
2. `make setup-local` — fix it. Boots Docker, starts `opengithub-postgres-test` container on :55433, runs migrations.
3. Re-run `make all && make test-e2e`. These automatically pick up the committed `.env.test`.

**The test DB URL is fixed:** `postgresql://opengithub:opengithub@localhost:55433/opengithub_test`. Do NOT invent alternative URLs — the watchdog wasted many iterations on `postgresql://postgres@localhost:55432/opengithub_identity_test`, which is wrong on every dimension. The correct values live in `docker-compose.test.yml` and `.env.test`.

New worktrees created via `./hack/create_worktree.sh` do full setup automatically: symlink `.env`/`.env.test`/`.mcp.json`, copy `.claude/` + `hack/` helpers, run `hack/setup_repo.sh` (which creates `.scratch/cargo-target` for per-worktree Cargo cache, writes `.envrc`, runs `npm ci` in `web/` if present, touches `.ralph-setup-done`), then run `make doctor`. On partial failure the worktree is removed automatically.

**Per-worktree Cargo cache (REQUIRED — replaces `/tmp/opengithub-cargo-target`):** activate with `export CARGO_TARGET_DIR="$PWD/.scratch/cargo-target"` (or `direnv allow`). The shared `/tmp/opengithub-cargo-target` path is legacy — it has no GC and reliably exhausts `/tmp` quota. Per-worktree paths are auto-cleaned when the worktree is removed and isolate lanes from each other's quota usage.

Tear down a worktree with `./hack/cleanup_worktree.sh [name]`.

## QA Sub-Phases (Progressive Disclosure)

Sub-phases are selected per feature category — not every feature runs all four.
The shell script (`qa-ralph.sh`) assembles the prompt from modules in `ralph/qa/`:
- `base.md` — always included (functional testing)
- `api.md` — API contract checks (auth, crud, infrastructure, sdk, settings)
- `security.md` — security probes (auth, crud, infrastructure)
- `a11y.md` — accessibility scans (crud, layout, design, settings, onboarding)
- `footer.md` — always included (record & fix, rules, checklist)

### Sub-Phase A: FUNCTIONAL
Automated and manual verification that the feature works as specified.
- Run `make test` (unit tests) and `make test-e2e` (Playwright E2E)
- Authenticate via Ever CLI: `ever start --url http://localhost:3015`
- Navigate to the feature page, `ever snapshot`, follow PRD acceptance criteria
- Test edge cases: empty inputs, rapid clicks, unexpected data
- If auth is required: set up Playwright auth fixture (`tests/e2e/auth.setup.ts`) — never skip

### Sub-Phase B: API CONTRACT
Verify every API endpoint for this feature returns correct shapes, status codes, and error formats.
- Discover endpoints per your stack (see `BUILD_GUIDE.md`). For Next.js App Router: `find src/app/api -name "route.ts" | sort`.
- Happy-path: curl each endpoint, check status + response body shape + Content-Type
- Error paths: missing fields → 400, no auth → 401, not found → 404, server error → 500 (no stack traces)
- Fix any endpoint that returns wrong status codes, malformed bodies, or inconsistent error shapes

### Sub-Phase C: SECURITY
Targeted checks for the most impactful vulnerabilities.
- **Auth bypass**: curl every endpoint without a token — must return 401, never 200 with data
- **Input sanitization**: probe SQL injection and XSS payloads — no errors leaked, no raw HTML reflected
- **CORS**: OPTIONS request with `Origin: https://evil.com` — `Access-Control-Allow-Origin` must not echo it back
- **Data exposure**: responses must never leak passwords, stack traces, or env variable values
- Fix all critical/major security findings before moving on

### Sub-Phase D: ACCESSIBILITY
axe-core scan + manual spot-checks for WCAG 2.1 AA compliance. JS/TS stacks only — for other stacks, do manual spot-checks only.
- `@axe-core/playwright` is pre-installed by `qa-ralph.sh` for JS stacks (no need to install inside Codex)
- Run axe scan via Playwright on every page touched by the feature (tags: wcag2a, wcag2aa, wcag21a, wcag21aa)
- Manual checks via Ever CLI: keyboard navigation, form labels, aria associations, color contrast
- Fix critical violations; log serious/moderate as known issues in qa-report.json

## How To Test (Quick Reference)

### Step 1: Automated regression (fast)
Run `make test-e2e` first. This catches obvious breakage in seconds.

### Step 2: Manual verification (Ever CLI)
- `ever snapshot` — see current page state
- `ever click <id>` — click elements
- `ever input <id> <text>` — fill inputs
- Read `ralph/ever-cli-reference.md` for full command reference

### Step 3: Real API testing
Test the clone's API directly:
```bash
curl -X POST http://localhost:3015/api/<endpoint> \
  -H "Authorization: Bearer <dev-api-key>" \
  -H "Content-Type: application/json" \
  -d '{"<request body>"}'
```
Check `build-progress.txt` or API routes for the dev API key and available endpoints.

### Step 4: SDK testing (if packages/sdk/ exists)
Test the SDK: import it, call the API, verify response.

## Architecture
Read `BUILD_GUIDE.md` in the repo root for stack-specific project structure. The stack was chosen during onboarding — check `ralph-config.json` for `language` and `stackProfile`.

General layout:
- Source code lives in `src/` (or language equivalent)
- Tests in `tests/` (unit) and `tests/e2e/` (E2E)
- Database schema and client in the db directory specified by the template
- `packages/sdk/` — SDK package (if applicable)

## Environment
- Cloud CLI configured via onboarding
- `.env` has credentials (DATABASE_URL, etc.)
- Dev server on port **3015**

## Bug Fixing Rules
- Fix bugs directly in source code
- Fix ALL bugs for a feature, then run `make check && make test` once before committing
- Commit fixes: `git commit -m "fix: <description>"`
- Push after every commit: `git push`
- **NEVER weaken or delete tests to make them pass.** Fix the code, not the test.
