#!/usr/bin/env bash
# Production/staging ECS deploy and rollback entrypoint for OpenGitHub.
# Safe by default for CI validation: set DRY_RUN=1 to print AWS/Docker commands
# without requiring live AWS credentials.
set -Eeuo pipefail

ACTION="${1:-deploy}"
ENVIRONMENT_ARG="${2:-${ENVIRONMENT:-${APP_ENV:-staging}}}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

: "${DEPLOY_HEALTH_TIMEOUT_SECONDS:=300}"
: "${DEPLOY_HEALTH_INTERVAL_SECONDS:=5}"
: "${DRY_RUN:=0}"
: "${MIGRATION_COMMAND:=sqlx migrate run --source crates/api/migrations}"
: "${RUN_DB_MIGRATIONS:=1}"
: "${MIGRATION_CONTAINER_NAME:=migration}"
: "${MIGRATION_ASSIGN_PUBLIC_IP:=DISABLED}"
: "${AWS_REGION:=${AWS_DEFAULT_REGION:-}}"
: "${GIT_SHA:=$(git -C "$ROOT_DIR" rev-parse --short=12 HEAD 2>/dev/null || echo unknown)}"

log() { printf '[deploy] %s\n' "$*" >&2; }
fatal() { printf '[deploy] ERROR: %s\n' "$*" >&2; exit 1; }

usage() {
  cat >&2 <<USAGE
Usage:
  scripts/deploy.sh deploy <staging|production>
  scripts/deploy.sh rollback <staging|production>

Required env for deploy:
  AWS_REGION AWS_ACCOUNT_ID ECR_API_REPOSITORY ECR_WEB_REPOSITORY ECR_MIGRATION_REPOSITORY ECS_CLUSTER
  ECS_API_SERVICE ECS_WEB_SERVICE ECS_API_TASK_FAMILY ECS_WEB_TASK_FAMILY MIGRATION_TASK_DEFINITION
  ECS_SUBNETS ECS_SECURITY_GROUPS
  API_URL WEB_URL
Optional:
  CLOUDFRONT_DISTRIBUTION_ID, RUN_DB_MIGRATIONS=0, MIGRATION_CONTAINER_NAME, MIGRATION_ASSIGN_PUBLIC_IP, DRY_RUN=1

Required env for rollback:
  AWS_REGION ECS_CLUSTER ECS_API_SERVICE ECS_WEB_SERVICE
  ROLLBACK_API_TASK_DEFINITION ROLLBACK_WEB_TASK_DEFINITION
USAGE
}

require_env() {
  local missing=0 name
  for name in "$@"; do
    if [[ -z "${!name:-}" ]]; then
      printf '[deploy] missing required env: %s\n' "$name" >&2
      missing=1
    fi
  done
  [[ "$missing" == 0 ]] || exit 2
}

run() {
  log "+ $*"
  if [[ "$DRY_RUN" == "1" ]]; then
    return 0
  fi
  "$@"
}

capture() {
  if [[ "$DRY_RUN" == "1" ]]; then
    printf 'dry-run-%s' "$GIT_SHA"
    return 0
  fi
  "$@"
}

aws_cmd() { run aws --region "$AWS_REGION" "$@"; }
aws_capture() { capture aws --region "$AWS_REGION" "$@"; }

wait_for_url() {
  local name="$1" url="$2" deadline=$((SECONDS + DEPLOY_HEALTH_TIMEOUT_SECONDS))
  if [[ "$DRY_RUN" == "1" ]]; then
    log "+ curl -fsS --max-time 5 $url"
    log "${name} healthy: ${url}"
    return 0
  fi
  while (( SECONDS < deadline )); do
    if curl -fsS --max-time 5 "$url" >/dev/null; then
      log "${name} healthy: ${url}"
      return 0
    fi
    sleep "$DEPLOY_HEALTH_INTERVAL_SECONDS"
  done
  fatal "${name} did not become healthy within ${DEPLOY_HEALTH_TIMEOUT_SECONDS}s: ${url}"
}

wait_for_services() {
  aws_cmd ecs wait services-stable --cluster "$ECS_CLUSTER" --services "$ECS_API_SERVICE" "$ECS_WEB_SERVICE"
}

image_uri() {
  local repo="$1"
  printf '%s.dkr.ecr.%s.amazonaws.com/%s:%s' "$AWS_ACCOUNT_ID" "$AWS_REGION" "$repo" "$GIT_SHA"
}

build_and_push() {
  local name="$1" dockerfile="$2" repo="$3" uri digest
  uri="$(image_uri "$repo")"
  log "building ${name} image git_sha=${GIT_SHA} image=${uri}"
  run docker build -f "$ROOT_DIR/$dockerfile" -t "$uri" "$ROOT_DIR"
  run docker push "$uri"
  digest="$(aws_capture ecr describe-images --repository-name "$repo" --image-ids imageTag="$GIT_SHA" --query 'imageDetails[0].imageDigest' --output text)"
  log "${name} image digest: ${digest}"
  printf '%s' "$uri"
}

render_task_def() {
  local family="$1" image="$2" out="$3"
  if [[ "$DRY_RUN" == "1" ]]; then
    cat >"$out" <<JSON
{
  "family": "${family}",
  "containerDefinitions": [
    { "name": "${family}", "image": "${image}" }
  ]
}
JSON
    return 0
  fi

  local raw
  raw="$(mktemp)"
  aws --region "$AWS_REGION" ecs describe-task-definition --task-definition "$family" --output json >"$raw"
  python3 - "$image" "$raw" >"$out" <<'PYJSON'
import json
import sys

image = sys.argv[1]
raw_path = sys.argv[2]
with open(raw_path, encoding="utf-8") as handle:
    payload = json.load(handle)["taskDefinition"]
for key in [
    "taskDefinitionArn",
    "revision",
    "status",
    "requiresAttributes",
    "compatibilities",
    "registeredAt",
    "registeredBy",
    "deregisteredAt",
]:
    payload.pop(key, None)
containers = payload.get("containerDefinitions", [])
if not containers:
    raise SystemExit("task definition has no containers")
containers[0]["image"] = image
json.dump(payload, sys.stdout)
PYJSON
  rm -f "$raw"
}

run_migration_task() {
  if [[ "$RUN_DB_MIGRATIONS" == "0" ]]; then
    log "skipping migrations because RUN_DB_MIGRATIONS=0"
    return 0
  fi
  require_env MIGRATION_TASK_DEFINITION ECS_SUBNETS ECS_SECURITY_GROUPS
  log "starting SQLx migration task before service rollout: ${MIGRATION_COMMAND}"
  local network_config task_arn status exit_code reason stopped_reason
  network_config="awsvpcConfiguration={subnets=[$ECS_SUBNETS],securityGroups=[$ECS_SECURITY_GROUPS],assignPublicIp=$MIGRATION_ASSIGN_PUBLIC_IP}"
  if [[ "$DRY_RUN" == "1" ]]; then
    log "+ aws --region $AWS_REGION ecs run-task --cluster $ECS_CLUSTER --task-definition $MIGRATION_TASK_DEFINITION --launch-type FARGATE --network-configuration $network_config"
    log "SQLx migrations succeeded: task=dry-run-${GIT_SHA} status=STOPPED exit=0"
    return 0
  fi
  task_arn="$(aws --region "$AWS_REGION" ecs run-task \
    --cluster "$ECS_CLUSTER" \
    --task-definition "$MIGRATION_TASK_DEFINITION" \
    --launch-type FARGATE \
    --network-configuration "$network_config" \
    --started-by "opengithub-deploy-migration" \
    --query 'tasks[0].taskArn' \
    --output text)"
  if [[ -z "$task_arn" || "$task_arn" == "None" ]]; then
    fatal "failed to start migration task"
  fi
  log "migration task started: $task_arn"
  log "waiting for migration task to stop; CloudWatch logs use the migration task log group and ecs/migration stream prefix"
  aws --region "$AWS_REGION" ecs wait tasks-stopped --cluster "$ECS_CLUSTER" --tasks "$task_arn"
  status="$(aws --region "$AWS_REGION" ecs describe-tasks --cluster "$ECS_CLUSTER" --tasks "$task_arn" --query "tasks[0].containers[?name=='$MIGRATION_CONTAINER_NAME'].lastStatus | [0]" --output text)"
  exit_code="$(aws --region "$AWS_REGION" ecs describe-tasks --cluster "$ECS_CLUSTER" --tasks "$task_arn" --query "tasks[0].containers[?name=='$MIGRATION_CONTAINER_NAME'].exitCode | [0]" --output text)"
  reason="$(aws --region "$AWS_REGION" ecs describe-tasks --cluster "$ECS_CLUSTER" --tasks "$task_arn" --query "tasks[0].containers[?name=='$MIGRATION_CONTAINER_NAME'].reason | [0]" --output text)"
  stopped_reason="$(aws --region "$AWS_REGION" ecs describe-tasks --cluster "$ECS_CLUSTER" --tasks "$task_arn" --query 'tasks[0].stoppedReason' --output text)"
  if [[ "$exit_code" == "0" ]]; then
    log "SQLx migrations succeeded: task=$task_arn status=$status exit=$exit_code"
    return 0
  fi
  log "SQLx migrations failed: task=$task_arn status=$status exit=$exit_code reason=${reason:-none} stoppedReason=${stopped_reason:-none}"
  fatal "blocking service rollout because database migrations did not complete successfully"
}

register_task() {
  local family="$1" image="$2" tmp arn
  tmp="$(mktemp)"
  render_task_def "$family" "$image" "$tmp"
  arn="$(aws_capture ecs register-task-definition --cli-input-json "file://$tmp" --query 'taskDefinition.taskDefinitionArn' --output text)"
  rm -f "$tmp"
  log "registered task definition family=${family} arn=${arn}"
  printf '%s' "$arn"
}

deploy() {
  require_env AWS_REGION AWS_ACCOUNT_ID ECR_API_REPOSITORY ECR_WEB_REPOSITORY ECR_MIGRATION_REPOSITORY ECS_CLUSTER ECS_API_SERVICE ECS_WEB_SERVICE ECS_API_TASK_FAMILY ECS_WEB_TASK_FAMILY MIGRATION_TASK_DEFINITION ECS_SUBNETS ECS_SECURITY_GROUPS API_URL WEB_URL
  log "starting deploy environment=${ENVIRONMENT_ARG} git_sha=${GIT_SHA} dry_run=${DRY_RUN}"
  log "logging in to ECR registry ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"
  if [[ "$DRY_RUN" == "1" ]]; then
    log "+ aws --region $AWS_REGION ecr get-login-password | docker login --username AWS --password-stdin ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"
  else
    aws --region "$AWS_REGION" ecr get-login-password | docker login --username AWS --password-stdin "${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"
  fi
  local api_image web_image migration_image api_task web_task
  api_image="$(build_and_push api Dockerfile.api "$ECR_API_REPOSITORY")"
  web_image="$(build_and_push web Dockerfile.web "$ECR_WEB_REPOSITORY")"
  migration_image="$(build_and_push migration Dockerfile.migration "$ECR_MIGRATION_REPOSITORY")"
  run_migration_task
  api_task="$(register_task "$ECS_API_TASK_FAMILY" "$api_image")"
  web_task="$(register_task "$ECS_WEB_TASK_FAMILY" "$web_image")"
  aws_cmd ecs update-service --cluster "$ECS_CLUSTER" --service "$ECS_API_SERVICE" --task-definition "$api_task"
  aws_cmd ecs update-service --cluster "$ECS_CLUSTER" --service "$ECS_WEB_SERVICE" --task-definition "$web_task"
  wait_for_services
  wait_for_url "api readiness" "${API_URL%/}/ready"
  wait_for_url "web health" "${WEB_URL%/}/healthz"
  if [[ -n "${CLOUDFRONT_DISTRIBUTION_ID:-}" ]]; then
    aws_cmd cloudfront create-invalidation --distribution-id "$CLOUDFRONT_DISTRIBUTION_ID" --paths '/*'
  fi
  log "deploy complete environment=${ENVIRONMENT_ARG} git_sha=${GIT_SHA} api_image=${api_image} web_image=${web_image}"
}

rollback() {
  require_env AWS_REGION ECS_CLUSTER ECS_API_SERVICE ECS_WEB_SERVICE ROLLBACK_API_TASK_DEFINITION ROLLBACK_WEB_TASK_DEFINITION
  log "starting rollback environment=${ENVIRONMENT_ARG} git_sha=${GIT_SHA} dry_run=${DRY_RUN}"
  aws_cmd ecs update-service --cluster "$ECS_CLUSTER" --service "$ECS_API_SERVICE" --task-definition "$ROLLBACK_API_TASK_DEFINITION"
  aws_cmd ecs update-service --cluster "$ECS_CLUSTER" --service "$ECS_WEB_SERVICE" --task-definition "$ROLLBACK_WEB_TASK_DEFINITION"
  wait_for_services
  if [[ -n "${API_URL:-}" ]]; then wait_for_url "api readiness" "${API_URL%/}/ready"; fi
  if [[ -n "${WEB_URL:-${APP_URL:-}}" ]]; then rollback_web_url="${WEB_URL:-${APP_URL}}"; wait_for_url "web health" "${rollback_web_url%/}/healthz"; fi
  log "rollback complete environment=${ENVIRONMENT_ARG} api_task=${ROLLBACK_API_TASK_DEFINITION} web_task=${ROLLBACK_WEB_TASK_DEFINITION}"
}

case "$ACTION" in
  deploy) deploy ;;
  rollback) rollback ;;
  -h|--help|help) usage; exit 0 ;;
  *) usage; fatal "unknown action: $ACTION" ;;
esac
