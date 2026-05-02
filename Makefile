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

# --- Guard: ensure onboarding has run ---
SETUP_DONE := $(wildcard .ralph-setup-done)

ifndef SETUP_DONE
$(error Stack not set up. Run onboarding first: /ralph-to-ralph-onboard)
endif

# --- Stack detection ---
HAS_WEB := $(wildcard web/package.json)
HAS_CARGO := $(wildcard Cargo.toml)

# Full validation: check + test
all: check test

# Static analysis: typecheck + lint
check: check-header typecheck lint

typecheck:
	@. ./hack/run_silent.sh && \
	if [ -n "$(HAS_CARGO)" ]; then \
	  run_silent "Cargo check passed" "cargo check --workspace --all-targets"; \
	fi && \
	if [ -n "$(HAS_WEB)" ]; then \
	  run_silent "Web typecheck passed" "cd web && npx tsc --noEmit"; \
	fi

lint:
	@. ./hack/run_silent.sh && \
	if [ -n "$(HAS_CARGO)" ]; then \
	  run_silent "Clippy passed" "cargo clippy --workspace --all-targets -- -D warnings"; \
	fi && \
	if [ -n "$(HAS_WEB)" ]; then \
	  run_silent "Biome passed" "cd web && npx biome check ."; \
	fi

fix:
	@if [ -n "$(HAS_CARGO)" ]; then cargo clippy --workspace --fix --allow-dirty --allow-staged -- -D warnings || true; fi
	@if [ -n "$(HAS_WEB)" ]; then cd web && npx biome check --write .; fi

format:
	@if [ -n "$(HAS_CARGO)" ]; then cargo fmt --all; fi
	@if [ -n "$(HAS_WEB)" ]; then cd web && npx biome format --write .; fi

# Unit tests
test: test-header
	@. ./hack/run_silent.sh && \
	if [ -n "$(HAS_CARGO)" ]; then \
	  run_silent "Cargo tests passed" "cargo test --workspace --all-targets"; \
	fi && \
	if [ -n "$(HAS_WEB)" ]; then \
	  run_silent_with_test_count "Web tests passed" "cd web && npx vitest run" "vitest"; \
	fi

# E2E tests (Playwright — only when web/ exists)
test-e2e:
	@if [ -n "$(HAS_WEB)" ]; then \
	  . ./hack/run_silent.sh && \
	  run_silent_with_test_count "E2E tests passed" "cd web && npx playwright test" "playwright"; \
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
	@if [ -n "$(HAS_WEB)" ]; then \
	  ( cargo watch -q -x 'run --bin api' & API_PID=$$! ; cd web && npm run dev ; kill $$API_PID 2>/dev/null ) ; \
	else \
	  cargo watch -q -x 'run --bin api' ; \
	fi

api-dev:
	cargo watch -q -x 'run --bin api'

web-dev:
	@if [ -n "$(HAS_WEB)" ]; then cd web && npm run dev; else echo "web/ not yet scaffolded"; exit 1; fi

# Production build
build:
	@if [ -n "$(HAS_CARGO)" ]; then cargo build --workspace --release; fi
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

# Clean build artifacts
clean:
	@if [ -n "$(HAS_CARGO)" ]; then cargo clean; fi
	@if [ -n "$(HAS_WEB)" ]; then cd web && rm -rf .next node_modules/.cache; fi

# Validate state files against JSON schemas
validate:
	@if [ -f scripts/validate-schemas.mjs ]; then node scripts/validate-schemas.mjs; else echo "(no validator yet)"; fi
