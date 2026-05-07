#!/bin/bash
# create_worktree.sh — Create a worktree under $HOME/wt/opengithub with .env symlinks
# Usage: ./hack/create_worktree.sh [worktree_name] [base_branch]
#   - no args: generated name, current branch as base
#   - one arg with '/': treated as remote branch (e.g. origin/feat-x)
#   - one arg: worktree name, base = current branch
#   - two args: worktree_name base_branch

set -e

generate_unique_name() {
    local adjectives=("swift" "bright" "clever" "smooth" "quick" "clean" "sharp" "neat" "cool" "fast")
    local nouns=("fix" "task" "work" "dev" "patch" "branch" "code" "build" "test" "run")
    local adj=${adjectives[$RANDOM % ${#adjectives[@]}]}
    local noun=${nouns[$RANDOM % ${#nouns[@]}]}
    local timestamp=$(date +%H%M)
    echo "${adj}_${noun}_${timestamp}"
}

if [ $# -eq 1 ] && [[ "$1" == */* ]]; then
    BASE_BRANCH="$1"
    WORKTREE_NAME="${1#*/}"
elif [ $# -ge 2 ]; then
    WORKTREE_NAME="$1"
    BASE_BRANCH="$2"
elif [ $# -eq 1 ]; then
    WORKTREE_NAME="$1"
    BASE_BRANCH=$(git branch --show-current)
else
    WORKTREE_NAME=$(generate_unique_name)
    BASE_BRANCH=$(git branch --show-current)
fi

REPO_BASE_NAME=$(basename "$(pwd)")
WORKTREES_BASE="${OPENGITHUB_WORKTREE_BASE:-$HOME/wt/${REPO_BASE_NAME}}"
WORKTREE_PATH="${WORKTREES_BASE}/${WORKTREE_NAME}"

echo "🌳 Creating worktree: ${WORKTREE_NAME}"
echo "📁 Location: ${WORKTREE_PATH}"

mkdir -p "$WORKTREES_BASE"

if [ -d "$WORKTREE_PATH" ]; then
    echo "❌ Error: Worktree directory already exists: $WORKTREE_PATH"
    exit 1
fi

if git worktree list --porcelain | grep -q "worktree $WORKTREE_PATH"; then
    echo "🧹 Cleaning up stale worktree registration..."
    git worktree remove --force "$WORKTREE_PATH" 2>/dev/null || git worktree prune
fi

echo "🔀 Creating from: ${BASE_BRANCH}"

LOCAL_BRANCH_NAME="$WORKTREE_NAME"

if [[ "$BASE_BRANCH" == */* ]]; then
    REMOTE_BRANCH="$BASE_BRANCH"
    BRANCH_NAME_ONLY="${BASE_BRANCH#*/}"
    echo "📥 Fetching remote branch..."
    git fetch "$(echo $BASE_BRANCH | cut -d'/' -f1)" "$BRANCH_NAME_ONLY" || git fetch --all

    if ! git show-ref --verify --quiet "refs/remotes/${REMOTE_BRANCH}"; then
        echo "❌ Error: Remote branch ${REMOTE_BRANCH} does not exist"
        exit 1
    fi

    if git show-ref --verify --quiet "refs/heads/${LOCAL_BRANCH_NAME}"; then
        git worktree add "$WORKTREE_PATH" "$LOCAL_BRANCH_NAME"
    else
        git worktree add -b "$LOCAL_BRANCH_NAME" "$WORKTREE_PATH" "$REMOTE_BRANCH"
        cd "$WORKTREE_PATH"
        git branch --set-upstream-to="$REMOTE_BRANCH" "$LOCAL_BRANCH_NAME" 2>/dev/null || true
        cd - > /dev/null
    fi
else
    if git show-ref --verify --quiet "refs/heads/${LOCAL_BRANCH_NAME}"; then
        git worktree add "$WORKTREE_PATH" "$LOCAL_BRANCH_NAME"
    else
        git worktree add -b "$LOCAL_BRANCH_NAME" "$WORKTREE_PATH" "$BASE_BRANCH"
    fi
fi

ORIGINAL_DIR=$(pwd)

# Skip Rust stack setup (already scaffolded in repo)
touch "${WORKTREE_PATH}/.ralph-setup-done"

# Copy .claude directory if present
if [ -d ".claude" ]; then
    echo "📋 Copying .claude/"
    cp -r .claude "$WORKTREE_PATH/"
fi

# Copy hack helpers (so worktree make targets work even if branch is older)
mkdir -p "$WORKTREE_PATH/hack"
[ -f hack/run_silent.sh ] && cp hack/run_silent.sh "$WORKTREE_PATH/hack/run_silent.sh"
[ -f hack/cargo_locked.sh ] && cp hack/cargo_locked.sh "$WORKTREE_PATH/hack/cargo_locked.sh"
[ -f hack/create_worktree.sh ] && cp hack/create_worktree.sh "$WORKTREE_PATH/hack/create_worktree.sh"

# Symlink .env / .env.* (skip .env.example, .git, node_modules)
while IFS= read -r env_file; do
    rel_path="${env_file#${ORIGINAL_DIR}/}"
    target_dir="${WORKTREE_PATH}/$(dirname "$rel_path")"
    mkdir -p "$target_dir"
    rm -f "${WORKTREE_PATH}/${rel_path}"
    ln -s "${env_file}" "${WORKTREE_PATH}/${rel_path}"
    echo "🔗 ${rel_path}"
done < <(find "${ORIGINAL_DIR}" -name ".env" -o -name ".env.*" 2>/dev/null | grep -v '\.env\.example' | grep -v '/\.git/' | grep -v '/node_modules/' | grep -v '/target/')

# Symlink .mcp.json if it exists
if [ -f "${ORIGINAL_DIR}/.mcp.json" ]; then
    rm -f "${WORKTREE_PATH}/.mcp.json"
    ln -s "${ORIGINAL_DIR}/.mcp.json" "${WORKTREE_PATH}/.mcp.json"
    echo "🔗 .mcp.json"
fi

cd - > /dev/null

# Probe verification setup so the user (or agent) knows immediately whether
# the new worktree can run tests. Non-fatal — just informational.
echo ""
echo "🩺 Running doctor in new worktree..."
(cd "$WORKTREE_PATH" && make -s doctor) || echo "(doctor reported issues — run 'make setup-local' inside the worktree to fix)"

echo ""
echo "✅ Worktree ready"
echo "📁 ${WORKTREE_PATH}"
echo "🔀 ${LOCAL_BRANCH_NAME}"
echo ""
echo "Next:"
echo "  cd ${WORKTREE_PATH}"
echo "  make setup-local   # if doctor reported missing pieces"
echo "  make all && make test-e2e"
echo ""
echo "To remove later:"
echo "  git worktree remove ${WORKTREE_PATH}"
echo "  git branch -D ${LOCAL_BRANCH_NAME}"
