#!/usr/bin/env bash
# Run Cargo commands with OpenGitHub's shared QA/build cache settings.
#
# Why this exists:
# - sccache deduplicates rustc outputs, but each worktree's target/ still grows
#   huge from deps, test binaries, linker outputs, and build-script artifacts.
# - A repo-scoped shared CARGO_TARGET_DIR keeps those artifacts out of every
#   worktree and lets close QA branches reuse them.
# - Cargo builds into one target dir should not run concurrently; serialize them
#   with lockf to avoid cache stomping/flaky QA.

set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: $0 <cargo-subcommand> [args...]" >&2
  exit 64
fi

cache_root="${OPENGITHUB_CACHE_ROOT:-$HOME/.cache/opengithub}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$cache_root/cargo-target}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
mkdir -p "$CARGO_TARGET_DIR" "$cache_root/locks"

lock_file="$cache_root/locks/cargo-target.lock"

if command -v lockf >/dev/null 2>&1; then
  exec lockf "$lock_file" cargo "$@"
elif command -v flock >/dev/null 2>&1; then
  exec flock "$lock_file" cargo "$@"
else
  echo "warning: neither lockf nor flock found; running cargo without shared-target lock" >&2
  exec cargo "$@"
fi
