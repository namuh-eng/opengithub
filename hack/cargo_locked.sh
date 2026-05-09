#!/usr/bin/env bash
# Run Cargo commands with OpenGitHub's worktree-aware target cache.
#
# Target dir resolution (first writable wins):
#   1. $CARGO_TARGET_DIR if already exported by the caller (their choice wins).
#   2. $REPO_ROOT/.scratch/cargo-target — per-worktree default. Isolates lanes,
#      avoids /tmp quota, survives when the shared cache is read-only (sandboxed
#      QA bundles mount $HOME/.cache read-only).
#   3. $HOME/.cache/opengithub/cargo-target — shared fallback for non-worktree
#      contexts. Serialized with flock against a shared lock file.
#
# If the resolved target dir is not writable we abort loudly. Earlier behavior
# was a Cargo "Read-only file system (os error 30)" panic deep in build-script
# land — see qa-opengithub-org-admin-003-mainqa-20260509 for the failure mode.

set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: $0 <cargo-subcommand> [args...]" >&2
  exit 64
fi

cache_root="${OPENGITHUB_CACHE_ROOT:-$HOME/.cache/opengithub}"

worktree_root=""
if command -v git >/dev/null 2>&1; then
  worktree_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
fi

target_is_writable() {
  local dir="$1"
  mkdir -p "$dir" 2>/dev/null || return 1
  [ -w "$dir" ]
}

if [ -n "${CARGO_TARGET_DIR:-}" ]; then
  if ! target_is_writable "$CARGO_TARGET_DIR"; then
    echo "error: CARGO_TARGET_DIR ($CARGO_TARGET_DIR) is not writable" >&2
    echo "       unset it to fall through to the per-worktree default, or pick a writable path" >&2
    exit 73
  fi
elif [ -n "$worktree_root" ] && target_is_writable "$worktree_root/.scratch/cargo-target"; then
  export CARGO_TARGET_DIR="$worktree_root/.scratch/cargo-target"
elif target_is_writable "$cache_root/cargo-target"; then
  export CARGO_TARGET_DIR="$cache_root/cargo-target"
else
  echo "error: no writable Cargo target dir found" >&2
  echo "       tried: \$CARGO_TARGET_DIR (unset)" >&2
  [ -n "$worktree_root" ] && echo "              $worktree_root/.scratch/cargo-target" >&2
  echo "              $cache_root/cargo-target" >&2
  echo "       set CARGO_TARGET_DIR to a writable path or run hack/setup_repo.sh" >&2
  exit 73
fi

export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"

# Only serialize when sharing the global cache. Per-worktree dirs are
# already isolated; locking would just slow them down.
if [ "$CARGO_TARGET_DIR" = "$cache_root/cargo-target" ]; then
  mkdir -p "$cache_root/locks"
  lock_file="$cache_root/locks/cargo-target.lock"
  if command -v lockf >/dev/null 2>&1; then
    exec lockf "$lock_file" cargo "$@"
  elif command -v flock >/dev/null 2>&1; then
    exec flock "$lock_file" cargo "$@"
  else
    echo "warning: neither lockf nor flock found; running cargo without shared-target lock" >&2
    exec cargo "$@"
  fi
else
  exec cargo "$@"
fi
