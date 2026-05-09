#!/usr/bin/env bash
# setup_repo.sh — Idempotent worktree/repo setup for opengithub.
#
# Run this in any worktree to prepare it for work. Safe to re-run.
# Sets up:
#   - .scratch/cargo-target          per-worktree Cargo target dir (avoids /tmp quota)
#   - .ralph-setup-done              ignored bootstrap marker (Makefile guard)
#   - web/node_modules               via `npm ci` from lockfile, only if web/ exists
#
# Cargo target convention:
#   We export `CARGO_TARGET_DIR=$PWD/.scratch/cargo-target` so cargo writes
#   build artifacts inside this worktree, not into a shared `/tmp` dir or the
#   global `$HOME/.cache/opengithub/cargo-target`. The per-worktree path:
#     - is auto-cleaned when the worktree is removed
#     - isolates lanes from each other's quota / lock contention
#   To activate it for your shell session: `export CARGO_TARGET_DIR="$PWD/.scratch/cargo-target"`
#   (or use direnv with the .envrc this script writes).

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "🔧 Setting up opengithub worktree at: $REPO_ROOT"

# 1) Per-worktree cargo target dir
mkdir -p .scratch/cargo-target
echo "  ✓ .scratch/cargo-target/ ready"

# 2) Bootstrap marker (Makefile guard)
if [ ! -f .ralph-setup-done ]; then
    echo "rust-platform-custom" > .ralph-setup-done
    echo "  ✓ .ralph-setup-done created"
else
    echo "  ✓ .ralph-setup-done already present"
fi

# 3) .envrc for direnv users (optional — direnv loads it automatically; harmless otherwise)
if [ ! -f .envrc ]; then
    cat > .envrc <<'EOF'
# Per-worktree Cargo target — keeps build artifacts inside this worktree
# (auto-cleaned on `git worktree remove`, isolates from other lanes).
export CARGO_TARGET_DIR="$PWD/.scratch/cargo-target"
EOF
    echo "  ✓ .envrc written (direnv users: run \`direnv allow\`)"
else
    echo "  ✓ .envrc already present"
fi

# 4) web/node_modules — fresh worktrees often miss this, causing
#    `Cannot find module '@playwright/test'`. Use `npm ci` (lockfile-faithful).
if [ -f web/package.json ]; then
    if [ ! -d web/node_modules ] || [ ! -d web/node_modules/@playwright/test ]; then
        echo "  → installing web deps via npm ci..."
        ( cd web && npm ci --no-audit --no-fund --silent )
        echo "  ✓ web/node_modules ready"
    else
        echo "  ✓ web/node_modules already present"
    fi
else
    echo "  ⚠ web/ not yet scaffolded (build loop creates it on first iteration)"
fi

echo ""
echo "✅ Setup complete."
echo ""
echo "Next:"
echo "  export CARGO_TARGET_DIR=\"\$PWD/.scratch/cargo-target\"   # or: direnv allow"
echo "  make doctor                                              # verify Docker/test DB"
echo "  make setup-local                                         # bring up test DB if needed"
echo "  make db-up-dev && make db-migrate-dev                    # bring up dev DB"
echo "  make all && make test-e2e                                # full verification"
