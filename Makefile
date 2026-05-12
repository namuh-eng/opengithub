# Ralph-to-Ralph Makefile — opengithub (Rust API + Next.js frontend)
#
# Stack:
#   - Rust workspace at repo root (crates/api/ — Axum + SQLx)
#   - Next.js frontend in web/ (created on demand by the build loop)
#
# Targets are dual-aware: Rust always runs; Next.js targets activate
# automatically once `web/package.json` exists.

.PHONY: check test test-e2e typecheck lint format fix all dev build clean validate
.PHONY: check-header test-header check-verbose test-verbose
.PHONY: db-generate db-migrate db-push api-dev web-dev
.PHONY: db-up-test db-down-test db-wait-test
.PHONY: db-up-dev db-down-dev db-wait-dev db-migrate-dev
.PHONY: doctor setup-local

# --- Guard: ensure onboarding has run ---
SETUP_DONE := $(wildcard .ralph-setup-done)

ifndef SETUP_DONE
$(error Stack not set up. Run onboarding first: /ralph-to-ralph-onboard)
endif

# --- Stack detection ---
HAS_WEB := $(wildcard web/package.json)
HAS_CARGO := $(wildcard Cargo.toml)
CARGO_LOCKED := ./hack/cargo_locked.sh
REAL_HOME ?= $(shell getent passwd $$(id -u) | cut -d: -f6)
PLAYWRIGHT_BROWSERS_PATH ?= $(REAL_HOME)/.cache/ms-playwright

# Container runtime plumbing. This repo is commonly verified on Ubuntu with
# rootless Podman exposed through the Docker-compatible API socket; do not rely
# on interactive shell aliases such as `alias docker=podman` being loaded.
CONTAINER_RUNTIME ?= $(shell if [ -S /run/user/$$(id -u)/podman/podman.sock ]; then echo podman; else echo docker; fi)
ifeq ($(CONTAINER_RUNTIME),podman)
DOCKER_HOST ?= unix:///run/user/$(shell id -u)/podman/podman.sock
DOCKER := env DOCKER_HOST=$(DOCKER_HOST) docker
else
DOCKER := docker
endif
COMPOSE_PROJECT_NAME ?= opengithub

# Full validation: check + test
all: check test

# Static analysis: typecheck + lint
check: check-header typecheck lint

typecheck:
	@. ./hack/run_silent.sh && \
	if [ -n "$(HAS_CARGO)" ]; then \
	  run_silent "Cargo check passed" "$(CARGO_LOCKED) check --workspace --all-targets"; \
	fi && \
	if [ -n "$(HAS_WEB)" ]; then \
	  run_silent "Web typecheck passed" "cd web && npx tsc --noEmit"; \
	fi

lint:
	@. ./hack/run_silent.sh && \
	if [ -n "$(HAS_CARGO)" ]; then \
	  run_silent "Clippy passed" "$(CARGO_LOCKED) clippy --workspace --all-targets -- -D warnings"; \
	fi && \
	if [ -n "$(HAS_WEB)" ]; then \
	  run_silent "Biome passed" "cd web && npx biome check ."; \
	fi

fix:
	@if [ -n "$(HAS_CARGO)" ]; then $(CARGO_LOCKED) clippy --workspace --fix --allow-dirty --allow-staged -- -D warnings || true; fi
	@if [ -n "$(HAS_WEB)" ]; then cd web && npx biome check --write .; fi

format:
	@if [ -n "$(HAS_CARGO)" ]; then cargo fmt --all; fi
	@if [ -n "$(HAS_WEB)" ]; then cd web && npx biome format --write .; fi

# Unit tests
test: test-header
	@. ./hack/run_silent.sh && \
	if [ -n "$(HAS_CARGO)" ]; then \
	  run_silent "Cargo tests passed" "$(CARGO_LOCKED) test --workspace --all-targets"; \
	fi && \
	if [ -n "$(HAS_WEB)" ]; then \
	  run_silent_with_test_count "Web tests passed" "cd web && TZ=UTC npx vitest run" "vitest"; \
	fi

# E2E tests (Playwright — only when web/ exists).
# Loads .env.test so TEST_DATABASE_URL / SESSION_* are wired for the test DB
# brought up by `make db-up-test`. Failing to bring up the DB is non-fatal
# here so the recipe can also run in CI where the DB is already provisioned.
test-e2e:
	@if [ -n "$(HAS_WEB)" ]; then \
	  set -a; [ -f .env.test ] && . ./.env.test; set +a; \
	  if [ -f .env.test ]; then $(MAKE) -s db-wait-test || true; fi; \
	  . ./hack/run_silent.sh && \
	  run_silent_with_test_count "E2E tests passed" "cd web && PLAYWRIGHT_BROWSERS_PATH='$(PLAYWRIGHT_BROWSERS_PATH)' npx playwright test" "playwright"; \
	else \
	  echo "(skipping e2e — web/ not yet scaffolded)"; \
	fi

# Headers
check-header:
	@sh -n ./hack/run_silent.sh || (echo "Shell script syntax error" && exit 1)
	@. ./hack/run_silent.sh && print_main_header "Running Checks"

test-header:
	@sh -n ./hack/run_silent.sh || (echo "Shell script syntax error" && exit 1)
	@. ./hack/run_silent.sh && print_main_header "Running Tests"

# Verbose versions (show full output)
check-verbose:
	@VERBOSE=1 $(MAKE) check

test-verbose:
	@VERBOSE=1 $(MAKE) test

# Dev: run API (with hot reload via cargo-watch) and Next.js together
dev:
	@if command -v cargo-watch >/dev/null 2>&1; then \
	  API_DEV_CMD='CARGO_TARGET_DIR="$${CARGO_TARGET_DIR:-$$PWD/.scratch/cargo-target}" CARGO_BUILD_JOBS="$${CARGO_BUILD_JOBS:-2}" CARGO_INCREMENTAL="$${CARGO_INCREMENTAL:-0}" cargo watch -q -x "run --bin api"'; \
	else \
	  API_DEV_CMD='$(CARGO_LOCKED) run --bin api'; \
	fi; \
	if [ -n "$(HAS_WEB)" ]; then \
	  ( sh -c "$$API_DEV_CMD" & API_PID=$$! ; cd web && npm run dev ; kill $$API_PID 2>/dev/null ) ; \
	else \
	  sh -c "$$API_DEV_CMD" ; \
	fi

api-dev:
	@if command -v cargo-watch >/dev/null 2>&1; then \
	  CARGO_TARGET_DIR="$${CARGO_TARGET_DIR:-$$PWD/.scratch/cargo-target}" CARGO_BUILD_JOBS="$${CARGO_BUILD_JOBS:-2}" CARGO_INCREMENTAL="$${CARGO_INCREMENTAL:-0}" cargo watch -q -x 'run --bin api'; \
	else \
	  $(CARGO_LOCKED) run --bin api; \
	fi

web-dev:
	@if [ -n "$(HAS_WEB)" ]; then cd web && npm run dev; else echo "web/ not yet scaffolded"; exit 1; fi

# Production build
build:
	@if [ -n "$(HAS_CARGO)" ]; then $(CARGO_LOCKED) build --workspace --release; fi
	@if [ -n "$(HAS_WEB)" ]; then cd web && npm run build; fi

# Database migrations (SQLx — wired once migrations exist in crates/api/migrations/)
db-generate:
	@echo "(implement once SQLx migrations live in crates/api/migrations/)"

db-migrate:
	@if [ -n "$(HAS_CARGO)" ]; then \
	  if command -v sqlx >/dev/null 2>&1; then \
	    sqlx migrate run --source crates/api/migrations; \
	  else \
	    echo "Install sqlx-cli first: cargo install sqlx-cli --no-default-features --features rustls,postgres"; \
	    exit 1; \
	  fi ; \
	fi

db-push: db-migrate

# Bring up the isolated Postgres for E2E / integration tests on :55433.
# Idempotent — safe to run repeatedly.
db-up-test:
	@if $(DOCKER) ps --filter "name=opengithub-postgres-test" --format '{{.Status}}' 2>/dev/null | grep -q "Up"; then \
	  echo "postgres-test container already running"; \
	else \
	  COMPOSE_PROJECT_NAME=$(COMPOSE_PROJECT_NAME) $(DOCKER) compose -f docker-compose.test.yml up -d; \
	fi
	@$(MAKE) -s db-wait-test

# Wait for the test DB to accept connections (used by db-up-test and test-e2e).
db-wait-test:
	@for i in $$(seq 1 60); do \
	  if $(DOCKER) exec opengithub-postgres-test pg_isready -U opengithub -d opengithub_test >/dev/null 2>&1; then \
	    exit 0; \
	  fi; \
	  if COMPOSE_PROJECT_NAME=$(COMPOSE_PROJECT_NAME) $(DOCKER) compose -f docker-compose.test.yml exec -T postgres-test pg_isready -U opengithub -d opengithub_test >/dev/null 2>&1; then \
	    exit 0; \
	  fi; \
	  sleep 1; \
	done; \
	echo "test postgres did not become ready on :55433 within 60s" >&2; \
	exit 1

# Tear down the test DB and drop the volume so the next run starts fresh.
db-down-test:
	@COMPOSE_PROJECT_NAME=$(COMPOSE_PROJECT_NAME) $(DOCKER) compose -f docker-compose.test.yml down -v

# --- Dev DB (persistent, port 55434) ---
# Separate from the test DB so `make db-down-test` never wipes your dev data.

db-up-dev:
	@COMPOSE_PROJECT_NAME=$(COMPOSE_PROJECT_NAME) $(DOCKER) compose -f docker-compose.dev.yml up -d
	@$(MAKE) -s db-wait-dev

db-wait-dev:
	@for i in $$(seq 1 60); do \
	  if COMPOSE_PROJECT_NAME=$(COMPOSE_PROJECT_NAME) $(DOCKER) compose -f docker-compose.dev.yml exec -T postgres-dev pg_isready -U opengithub -d opengithub_dev >/dev/null 2>&1; then \
	    exit 0; \
	  fi; \
	  sleep 1; \
	done; \
	echo "dev postgres did not become ready on :55434 within 60s" >&2; \
	exit 1

# Stop the dev DB but preserve the volume — data survives across restarts.
db-down-dev:
	@COMPOSE_PROJECT_NAME=$(COMPOSE_PROJECT_NAME) $(DOCKER) compose -f docker-compose.dev.yml down

# Apply migrations to the dev DB. Requires sqlx-cli.
db-migrate-dev:
	@if ! command -v sqlx >/dev/null 2>&1; then \
	  echo "Install sqlx-cli first: cargo install sqlx-cli --no-default-features --features rustls,postgres"; \
	  exit 1; \
	fi
	@DATABASE_URL=postgresql://opengithub:opengithub@localhost:55434/opengithub_dev sqlx migrate run --source crates/api/migrations

# Diagnose local dev/test setup. Run this in any worktree to know what's
# missing. Exits 0 if healthy, non-zero with actionable guidance if not.
# Agents: run `make doctor` before claiming verification is complete.
doctor:
	@ok=1; \
	if $(DOCKER) info >/dev/null 2>&1; then \
	  printf "  \033[32m✓\033[0m Container runtime reachable ($(CONTAINER_RUNTIME))\n"; \
	else \
	  printf "  \033[31m✗\033[0m Container runtime not reachable — run: make setup-local\n"; ok=0; \
	fi; \
	if $(DOCKER) ps --filter "name=opengithub-postgres-test" --format '{{.Status}}' 2>/dev/null | grep -q "Up"; then \
	  printf "  \033[32m✓\033[0m postgres-test container up\n"; \
	else \
	  printf "  \033[31m✗\033[0m postgres-test container not running — run: make setup-local\n"; ok=0; \
	fi; \
	if command -v pg_isready >/dev/null 2>&1 && pg_isready -h localhost -p 55433 -U opengithub -d opengithub_test >/dev/null 2>&1; then \
	  printf "  \033[32m✓\033[0m Postgres reachable on :55433\n"; \
	elif $(DOCKER) exec opengithub-postgres-test pg_isready -U opengithub -d opengithub_test >/dev/null 2>&1; then \
	  printf "  \033[32m✓\033[0m Postgres reachable inside opengithub-postgres-test\n"; \
	elif COMPOSE_PROJECT_NAME=$(COMPOSE_PROJECT_NAME) $(DOCKER) compose -f docker-compose.test.yml exec -T postgres-test pg_isready -U opengithub -d opengithub_test >/dev/null 2>&1; then \
	  printf "  \033[32m✓\033[0m Postgres reachable inside compose postgres-test\n"; \
	else \
	  printf "  \033[31m✗\033[0m Postgres not accepting connections on :55433 — run: make setup-local\n"; ok=0; \
	fi; \
	if [ -f .env.test ]; then \
	  printf "  \033[32m✓\033[0m .env.test present\n"; \
	else \
	  printf "  \033[31m✗\033[0m .env.test missing — checkout from main, it is committed\n"; ok=0; \
	fi; \
	if [ -e .env ]; then \
	  printf "  \033[32m✓\033[0m .env present\n"; \
	else \
	  printf "  \033[33m!\033[0m .env missing (only needed for live OAuth/AWS dev, not for tests)\n"; \
	fi; \
	if [ -n "$(HAS_CARGO)" ]; then \
	  effective_target="$${CARGO_TARGET_DIR:-$$PWD/.scratch/cargo-target}"; \
	  if mkdir -p "$$effective_target" 2>/dev/null && [ -w "$$effective_target" ]; then \
	    printf "  \033[32m✓\033[0m Cargo target dir writable ($$effective_target)\n"; \
	  else \
	    printf "  \033[31m✗\033[0m Cargo target dir not writable ($$effective_target) — set CARGO_TARGET_DIR to a writable path (e.g. \$$PWD/.scratch/cargo-target)\n"; ok=0; \
	  fi; \
	fi; \
	if [ "$$ok" = "1" ]; then \
	  printf "\n\033[32mLocal verification stack is healthy.\033[0m Run: make all && make test-e2e\n"; \
	else \
	  printf "\n\033[31mSome required checks failed.\033[0m Run: make setup-local\n"; exit 1; \
	fi

# One-shot bring-up of the local test DB. Idempotent — safe to rerun.
# Starts the configured container runtime, brings up postgres-test, runs
# migrations, then runs `make doctor` to confirm.
setup-local:
	@if ! $(DOCKER) info >/dev/null 2>&1; then \
	  case "$$(uname -s)" in \
	    Darwin) echo "Starting Docker Desktop..."; open -a Docker 2>/dev/null || true ;; \
	    Linux) echo "Starting Podman/Docker socket if available..."; systemctl --user start podman.socket 2>/dev/null || sudo systemctl start docker 2>/dev/null || true ;; \
	    *) echo "Unknown OS; please start $(CONTAINER_RUNTIME) manually" ;; \
	  esac; \
	  for i in $$(seq 1 60); do \
	    if $(DOCKER) info >/dev/null 2>&1; then break; fi; \
	    sleep 2; \
	  done; \
	  if ! $(DOCKER) info >/dev/null 2>&1; then \
	    echo "Container runtime did not become ready within 120s. Start $(CONTAINER_RUNTIME) manually and rerun." && exit 1; \
	  fi; \
	fi
	@echo "Bringing up postgres-test container..."
	@$(MAKE) -s db-up-test
	@echo "Applying migrations (if sqlx-cli installed)..."
	@if command -v sqlx >/dev/null 2>&1 && [ -d crates/api/migrations ]; then \
	  DATABASE_URL=postgresql://opengithub:opengithub@localhost:55433/opengithub_test sqlx migrate run --source crates/api/migrations || true; \
	else \
	  echo "(sqlx-cli not installed or no migrations yet — skipping)"; \
	fi
	@$(MAKE) -s doctor

# Clean build artifacts
clean:
	@if [ -n "$(HAS_CARGO)" ]; then CARGO_TARGET_DIR="$${CARGO_TARGET_DIR:-$$PWD/.scratch/cargo-target}" cargo clean; fi
	@if [ -n "$(HAS_WEB)" ]; then cd web && rm -rf .next node_modules/.cache; fi

# Validate state files against JSON schemas
validate:
	@if [ -f scripts/validate-schemas.mjs ]; then node scripts/validate-schemas.mjs; else echo "(no validator yet)"; fi
