#!/usr/bin/env bash
set -euo pipefail

export TEST_DATABASE_URL="${TEST_DATABASE_URL:-${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/opengithub_identity_test}}"
export DB_SSL="${DB_SSL:-false}"

cargo test --test api_pull_request_diff_review_contract -- --nocapture
