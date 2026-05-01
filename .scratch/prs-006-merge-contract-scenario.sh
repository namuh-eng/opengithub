#!/usr/bin/env bash
set -euo pipefail

export TEST_DATABASE_URL="${TEST_DATABASE_URL:-${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/opengithub_identity_test}}"
export DB_SSL="${DB_SSL:-false}"

cargo test --test api_pull_request_detail_contract pull_request_mergeability_uses_repository_policy_and_branch_rules -- --nocapture
