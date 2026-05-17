#!/usr/bin/env bash
# Production deployment gate for OpenGitHub.
# The actual ECS service update may be performed by CI/CD or IaC; this script
# records the rollout health contract and can be used as the post-deploy waiter.
set -euo pipefail

API_URL="${API_URL:-}"
WEB_URL="${WEB_URL:-${APP_URL:-}}"
TIMEOUT_SECONDS="${DEPLOY_HEALTH_TIMEOUT_SECONDS:-300}"
INTERVAL_SECONDS="${DEPLOY_HEALTH_INTERVAL_SECONDS:-5}"

if [[ -z "$API_URL" || -z "$WEB_URL" ]]; then
  echo "Usage: API_URL=https://api.example WEB_URL=https://app.example $0" >&2
  echo "Requires API /ready and web /healthz to pass before deployment is considered healthy." >&2
  exit 2
fi

wait_for() {
  local name="$1"
  local url="$2"
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  while (( SECONDS < deadline )); do
    if curl -fsS --max-time 5 "$url" >/dev/null; then
      echo "${name} healthy: ${url}"
      return 0
    fi
    sleep "$INTERVAL_SECONDS"
  done
  echo "${name} did not become healthy within ${TIMEOUT_SECONDS}s: ${url}" >&2
  return 1
}

wait_for "api readiness" "${API_URL%/}/ready"
wait_for "web health" "${WEB_URL%/}/healthz"
