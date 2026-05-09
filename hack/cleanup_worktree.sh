#!/usr/bin/env bash
# cleanup_worktree.sh — Remove an opengithub git worktree and its branch.
#
# Usage: ./hack/cleanup_worktree.sh [worktree_name]
#   - no args: lists worktrees under $HOME/wt/opengithub
#   - one arg: removes the named worktree (and prompts to delete its branch)

set -euo pipefail

REPO_BASE_NAME="$(basename "$(git rev-parse --show-toplevel)")"
WORKTREE_BASE_DIR="${OPENGITHUB_WORKTREE_BASE:-$HOME/wt/${REPO_BASE_NAME}}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

list_worktrees() {
    echo -e "${YELLOW}Worktrees under ${WORKTREE_BASE_DIR}:${NC}"
    git worktree list | grep -E "^${WORKTREE_BASE_DIR}" || {
        echo "  (none)"
        return 1
    }
}

cleanup_worktree() {
    local name="$1"
    local path="${WORKTREE_BASE_DIR}/${name}"

    if ! git worktree list | grep -q "$path"; then
        echo -e "${RED}Error: no worktree at $path${NC}"
        echo ""
        list_worktrees || true
        exit 1
    fi

    echo -e "${YELLOW}Removing worktree: $path${NC}"

    # Drop scratch caches before git removes the dir (faster, gives clear feedback)
    if [ -d "$path/.scratch" ]; then
        echo "  → removing .scratch/ (cargo-target, tmp)"
        rm -rf "$path/.scratch"
    fi

    if git worktree remove --force "$path"; then
        echo -e "${GREEN}  ✓ worktree removed${NC}"
    else
        echo -e "${RED}  ✗ git worktree remove failed${NC}"
        echo "  Try manually:"
        echo "    rm -rf $path && git worktree prune"
        exit 1
    fi

    echo ""
    read -p "Delete branch '${name}'? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if git branch -D "$name" 2>/dev/null; then
            echo -e "${GREEN}  ✓ branch deleted${NC}"
        else
            echo -e "${YELLOW}  ! branch '${name}' did not exist or could not be deleted${NC}"
        fi
    else
        echo "  branch '${name}' kept"
    fi

    git worktree prune
    echo ""
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

if [ $# -eq 0 ]; then
    list_worktrees || exit 1
    echo ""
    echo "Usage: $0 <worktree_name>"
    exit 0
fi

cleanup_worktree "$1"
