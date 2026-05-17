#!/bin/bash
# AWS infrastructure preflight for OpenGitHub.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TF_DIR="$ROOT_DIR/infra/terraform"
CMD="${1:-validate}"

terraform_bin() {
  if command -v terraform >/dev/null 2>&1; then
    echo terraform
  elif command -v tofu >/dev/null 2>&1; then
    echo tofu
  else
    return 1
  fi
}

static_checks() {
  test -f "$TF_DIR/versions.tf"
  test -f "$TF_DIR/main.tf"
  test -f "$TF_DIR/variables.tf"
  test -f "$TF_DIR/outputs.tf"
  grep -q 'engine_version *= *"16"' "$TF_DIR/main.tf"
  grep -q 'publicly_accessible *= *false' "$TF_DIR/main.tf"
  grep -q 'aws_ecs_service' "$TF_DIR/main.tf"
  grep -q 'aws_cloudfront_distribution' "$TF_DIR/main.tf"
  grep -q 'aws_ses_domain_identity' "$TF_DIR/main.tf"
  grep -q 'aws_ecr_repository' "$TF_DIR/main.tf"
  grep -q 'aws_ecs_task_definition" "migration' "$TF_DIR/main.tf"
  grep -q '"sqlx", "migrate", "run", "--source", "crates/api/migrations"' "$TF_DIR/main.tf"
}

case "$CMD" in
  validate)
    static_checks
    if TF_BIN="$(terraform_bin)"; then
      cd "$TF_DIR"
      "$TF_BIN" fmt -check -recursive
      "$TF_BIN" init -backend=false -input=false
      "$TF_BIN" validate
    else
      echo "Terraform/OpenTofu not installed; completed static IaC checks only." >&2
      echo "Install Terraform >=1.6 or OpenTofu and rerun: ./scripts/preflight.sh validate" >&2
    fi
    ;;
  plan|apply)
    TF_BIN="$(terraform_bin)" || { echo "Terraform/OpenTofu is required for $CMD." >&2; exit 1; }
    cd "$TF_DIR"
    "$TF_BIN" init
    "$TF_BIN" "$CMD" "${@:2}"
    ;;
  *)
    echo "Usage: $0 [validate|plan|apply] [terraform args...]" >&2
    exit 2
    ;;
esac
