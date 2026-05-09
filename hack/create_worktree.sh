#!/usr/bin/env bash
# create_worktree.sh — Create an opengithub git worktree and run full setup.
#
# Usage: ./hack/create_worktree.sh [worktree_name] [base_branch]
#   - no args            generated name; current branch as base
#   - one arg with '/'   treated as a remote ref (e.g. origin/feat-x)
#   - one arg            worktree name; current branch as base
#   - two args           worktree_name base_branch
#
# Worktrees land under $HOME/wt/opengithub/ (override with OPENGITHUB_WORKTREE_BASE).
#
# After creating the worktree the script:
#   - copies .claude/ and hack/ helpers (so make targets work even on older branches)
#   - symlinks .env, .env.test, .mcp.json from the source repo
#   - runs hack/setup_repo.sh inside the new worktree
#       → creates .scratch/cargo-target (per-worktree Cargo target dir)
#       → writes .envrc that exports CARGO_TARGET_DIR
#       → runs `npm ci` in web/ if web/package.json exists
#   - runs `make doctor` so you immediately know if local verification is healthy
#
# If setup fails, the worktree is removed automatically — you never end up
# with a half-configured worktree.

set -euo pipefail

generate_unique_name() {
    local adjectives=("swift" "bright" "clever" "smooth" "quick" "clean" "sharp" "neat" "cool" "fast")
    local nouns=("fix" "task" "work" "dev" "patch" "branch" "code" "build" "test" "run")
    local adj=${adjectives[$RANDOM % ${#adjectives[@]}]}
    local noun=${nouns[$RANDOM % ${#nouns[@]}]}
    local timestamp=$(date +%H%M)
    echo "${adj}_${noun}_${timestamp}"
}

# --- parse args ---
if [ $# -eq 1 ] && [[ "$1" == */* ]]; then
    BASE_BRANCH="$1"
    WORKTREE_NAME="${1#*/}"
elif [ $# -ge 2 ]; then
    WORKTREE_NAME="$1"
    BASE_BRANCH="$2"
elif [ $# -eq 1 ]; then
    WORKTREE_NAME="$1"
    BASE_BRANCH="$(git branch --show-current)"
else
    WORKTREE_NAME="$(generate_unique_name)"
    BASE_BRANCH="$(git branch --show-current)"
fi

REPO_BASE_NAME="$(basename "$(git rev-parse --show-toplevel)")"
WORKTREES_BASE="${OPENGITHUB_WORKTREE_BASE:-$HOME/wt/${REPO_BASE_NAME}}"
WORKTREE_PATH="${WORKTREES_BASE}/${WORKTREE_NAME}"
ORIGINAL_DIR="$(git rev-parse --show-toplevel)"

echo "🌳 Worktree: ${WORKTREE_NAME}"
echo "📁 Path:     ${WORKTREE_PATH}"
echo "🔀 Base:     ${BASE_BRANCH}"

mkdir -p "$WORKTREES_BASE"

if [ -d "$WORKTREE_PATH" ]; then
    echo "❌ Worktree directory already exists: $WORKTREE_PATH"
    exit 1
fi

if git worktree list --porcelain | grep -q "worktree $WORKTREE_PATH"; then
    echo "🧹 Pruning stale worktree registration..."
    git worktree remove --force "$WORKTREE_PATH" 2>/dev/null || git worktree prune
fi

LOCAL_BRANCH_NAME="$WORKTREE_NAME"

# --- create the worktree ---
if [[ "$BASE_BRANCH" == */* ]]; then
    REMOTE_BRANCH="$BASE_BRANCH"
    BRANCH_NAME_ONLY="${BASE_BRANCH#*/}"
    echo "📥 Fetching remote branch..."
    git fetch "$(echo "$BASE_BRANCH" | cut -d'/' -f1)" "$BRANCH_NAME_ONLY" || git fetch --all

    if ! git show-ref --verify --quiet "refs/remotes/${REMOTE_BRANCH}"; then
        echo "❌ Remote branch ${REMOTE_BRANCH} does not exist"
        exit 1
    fi

    if git show-ref --verify --quiet "refs/heads/${LOCAL_BRANCH_NAME}"; then
        git worktree add "$WORKTREE_PATH" "$LOCAL_BRANCH_NAME"
    else
        git worktree add -b "$LOCAL_BRANCH_NAME" "$WORKTREE_PATH" "$REMOTE_BRANCH"
        ( cd "$WORKTREE_PATH" && git branch --set-upstream-to="$REMOTE_BRANCH" "$LOCAL_BRANCH_NAME" 2>/dev/null || true )
    fi
else
    if git show-ref --verify --quiet "refs/heads/${LOCAL_BRANCH_NAME}"; then
        git worktree add "$WORKTREE_PATH" "$LOCAL_BRANCH_NAME"
    else
        git worktree add -b "$LOCAL_BRANCH_NAME" "$WORKTREE_PATH" "$BASE_BRANCH"
    fi
fi

# --- abort + cleanup helper for partial setup failures ---
abort_with_cleanup() {
    local reason="$1"
    echo "❌ ${reason}"
    echo "🧹 Removing partial worktree..."
    git worktree remove --force "$WORKTREE_PATH" 2>/dev/null || true
    git branch -D "$LOCAL_BRANCH_NAME" 2>/dev/null || true
    exit 1
}

# --- copy .claude/ ---
if [ -d "${ORIGINAL_DIR}/.claude" ]; then
    echo "📋 Copying .claude/"
    cp -r "${ORIGINAL_DIR}/.claude" "$WORKTREE_PATH/"
fi

# --- copy hack helpers (so make targets work even on older branches) ---
mkdir -p "$WORKTREE_PATH/hack"
for f in run_silent.sh cargo_locked.sh create_worktree.sh setup_repo.sh cleanup_worktree.sh; do
    if [ -f "${ORIGINAL_DIR}/hack/$f" ]; then
        cp "${ORIGINAL_DIR}/hack/$f" "$WORKTREE_PATH/hack/$f"
    fi
done

# --- symlink env files (.env, .env.test, etc — never .env.example) ---
while IFS= read -r env_file; do
    rel_path="${env_file#${ORIGINAL_DIR}/}"
    target_dir="${WORKTREE_PATH}/$(dirname "$rel_path")"
    mkdir -p "$target_dir"
    rm -f "${WORKTREE_PATH}/${rel_path}"
    ln -s "${env_file}" "${WORKTREE_PATH}/${rel_path}"
    echo "🔗 ${rel_path}"
done < <(find "${ORIGINAL_DIR}" -maxdepth 3 \( -name ".env" -o -name ".env.*" \) 2>/dev/null \
            | grep -v '\.env\.example' | grep -v '/\.git/' | grep -v '/node_modules/' | grep -v '/target/' | grep -v '/\.scratch/')

# --- symlink .mcp.json ---
if [ -f "${ORIGINAL_DIR}/.mcp.json" ]; then
    rm -f "${WORKTREE_PATH}/.mcp.json"
    ln -s "${ORIGINAL_DIR}/.mcp.json" "${WORKTREE_PATH}/.mcp.json"
    echo "🔗 .mcp.json"
fi

# --- run setup_repo.sh inside the worktree ---
echo ""
echo "🔧 Running hack/setup_repo.sh in worktree..."
if ! ( cd "$WORKTREE_PATH" && ./hack/setup_repo.sh ); then
    abort_with_cleanup "Setup failed inside worktree."
fi

# --- run doctor (informational; non-fatal) ---
echo ""
echo "🩺 Running make doctor in worktree..."
( cd "$WORKTREE_PATH" && make -s doctor ) || \
    echo "(doctor reported issues — run \`make setup-local\` inside the worktree to fix)"

echo ""
echo "✅ Worktree ready"
echo "📁 ${WORKTREE_PATH}"
echo "🔀 ${LOCAL_BRANCH_NAME}"
echo ""
echo "Next:"
echo "  cd ${WORKTREE_PATH}"
echo "  export CARGO_TARGET_DIR=\"\$PWD/.scratch/cargo-target\"   # or: direnv allow"
echo "  make all && make test-e2e"
echo ""
echo "To remove later:"
echo "  ./hack/cleanup_worktree.sh ${LOCAL_BRANCH_NAME}"
